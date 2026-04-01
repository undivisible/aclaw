//! Agent loop — the core execution engine.
//! Processes incoming messages, calls LLM, executes tools, sends responses.
//! Supports progress callbacks for real-time feedback.

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::sync::RwLock;

use crate::agent::hooks::{run_post_hooks, run_pre_hooks, HookDecision, ToolHook};
use crate::agent::mode::{is_approval, is_rejection, AgentMode, NullChannel, PendingPlan, PendingPlans};
use crate::channels::{Channel, IncomingMessage, OutgoingMessage};
use crate::cost::{CostTracker, TokenUsage};
use crate::memory::MemoryBackend;
use crate::providers::{ChatMessage, ChatRequest, Provider};
use crate::skills;
use crate::tools::Tool;

/// Warn the LLM after this many identical tool calls
const LOOP_WARN_THRESHOLD: usize = 5;
/// Hard stop after this many identical consecutive tool calls
const LOOP_BREAK_THRESHOLD: usize = 8;

/// Agent execution state machine
#[derive(Debug, Clone, PartialEq)]
enum AgentState {
    /// Planning: Haiku analyzes the request, makes a plan (temp 0.8)
    Planning,
    /// Executing: Sonnet follows the plan, calls tools (temp 0.2)
    Executing,
    /// Summarizing: Haiku compacts results into final response (temp 0.7)
    Summarizing,
    /// Direct: Simple query, no planning needed — use main model (temp 0.7)
    Direct,
}

pub struct AgentRunner {
    provider: Arc<dyn Provider>,
    /// Hot-reloadable tools list — shared with watcher + create_tool
    pub tools: Arc<RwLock<Vec<Arc<dyn Tool>>>>,
    memory: Arc<dyn MemoryBackend>,
    /// Hot-reloadable system prompt — updated when MEMORY.md / context files change
    pub system_prompt: Arc<RwLock<String>>,
    model: std::sync::RwLock<String>,
    /// The model as originally configured — used for reset-to-default
    default_model: String,
    workspace: PathBuf,
    /// Hot-reloadable skills — re-discovered when skills/ dir changes
    pub skills: Arc<RwLock<Vec<skills::Skill>>>,
    cost_tracker: Arc<CostTracker>,
    /// Steering messages — injected into the loop between rounds
    pub steering_queue: Arc<std::sync::Mutex<Vec<String>>>,
    /// Agent limits and model preferences
    pub agent_config: crate::config::AgentConfig,
    /// Execution mode (Auto / Coding / BypassPermissions / Swarm)
    mode: Arc<std::sync::RwLock<AgentMode>>,
    /// Pending plans awaiting user approval, keyed by chat_id
    pending_plans: PendingPlans,
    /// Registered tool hooks (PreToolUse / PostToolUse)
    hooks: Arc<std::sync::RwLock<Vec<Arc<dyn ToolHook>>>>,
}

impl AgentRunner {
    pub fn new(
        provider: Arc<dyn Provider>,
        tools: Vec<Arc<dyn Tool>>,
        memory: Arc<dyn MemoryBackend>,
        system_prompt: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        let model_str = model.into();
        Self {
            provider,
            tools: Arc::new(RwLock::new(tools)),
            memory,
            system_prompt: Arc::new(RwLock::new(system_prompt.into())),
            default_model: model_str.clone(),
            model: std::sync::RwLock::new(model_str),
            workspace: PathBuf::from("."),
            skills: Arc::new(RwLock::new(Vec::new())),
            cost_tracker: Arc::new(CostTracker::new()),
            steering_queue: Arc::new(std::sync::Mutex::new(Vec::new())),
            agent_config: crate::config::AgentConfig::default(),
            mode: Arc::new(std::sync::RwLock::new(AgentMode::default())),
            pending_plans: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            hooks: Arc::new(std::sync::RwLock::new(Vec::new())),
        }
    }

    pub fn with_config(mut self, config: crate::config::AgentConfig) -> Self {
        self.agent_config = config;
        self
    }

    pub fn with_mode(self, mode: AgentMode) -> Self {
        *self.mode.write().unwrap() = mode;
        self
    }

    /// Read the current execution mode.
    pub fn get_mode(&self) -> AgentMode {
        self.mode.read().unwrap().clone()
    }

    /// Set the execution mode at runtime (used by ModeSwitchTool).
    pub fn set_mode(&self, mode: AgentMode) {
        *self.mode.write().unwrap() = mode;
    }

    /// Arc handle to the mode lock — pass to ModeSwitchTool.
    pub fn mode_handle(&self) -> Arc<std::sync::RwLock<AgentMode>> {
        self.mode.clone()
    }

    /// Register a tool hook (PreToolUse / PostToolUse).
    pub fn add_hook(&self, hook: Arc<dyn ToolHook>) {
        self.hooks.write().unwrap().push(hook);
    }

    /// Queue a steering message to inject into the current agent loop
    pub fn steer(&self, message: String) {
        self.steering_queue.lock().unwrap().push(message);
    }

    pub fn with_workspace(mut self, workspace: PathBuf) -> Self {
        self.workspace = workspace;
        self
    }

    pub async fn with_skills(self, skills: Vec<skills::Skill>) -> Self {
        *self.skills.write().await = skills;
        self
    }

    /// Get cost tracker reference (for ClaudeUsageTool)
    pub fn cost_tracker(&self) -> Arc<CostTracker> {
        self.cost_tracker.clone()
    }

    /// Get cost summary
    pub async fn get_cost_summary(&self) -> crate::cost::CostSummary {
        self.cost_tracker.summary().await
    }

    /// Get current model name
    pub fn get_model(&self) -> String {
        self.model.read().unwrap().clone()
    }

    /// Get the originally configured model (before any runtime overrides)
    pub fn get_default_model(&self) -> &str {
        &self.default_model
    }

    /// Switch model at runtime
    pub fn set_model(&self, model: impl Into<String>) {
        *self.model.write().unwrap() = model.into();
    }

    /// Reset model to originally configured default
    pub fn reset_model(&self) {
        *self.model.write().unwrap() = self.default_model.clone();
    }

    /// List available tools
    pub async fn list_tools(&self) -> Vec<String> {
        self.tools
            .read()
            .await
            .iter()
            .map(|t| t.name().to_string())
            .collect()
    }

    /// Add a tool at runtime (for late-binding tools like session_status)
    pub async fn add_tool(&self, tool: Arc<dyn Tool>) {
        self.tools.write().await.push(tool);
    }

    /// Deploy parallel headless agent workers for a coding swarm.
    ///
    /// Each task runs in an isolated `AgentRunner` context (shared provider + tools,
    /// separate conversation history). Workers execute `parallelism` at a time.
    /// Returns `(task, result)` pairs in completion order.
    pub async fn deploy_coding_swarm(
        self: Arc<Self>,
        tasks: Vec<String>,
        base_chat_id: &str,
        parallelism: usize,
    ) -> Vec<(String, String)> {
        let parallelism = parallelism.max(1);
        let mut all_results = Vec::new();

        for (chunk_idx, chunk) in tasks.chunks(parallelism).enumerate() {
            let handles: Vec<_> = chunk
                .iter()
                .enumerate()
                .map(|(i, task)| {
                    let runner = self.clone();
                    let chat_id = format!("{}_sw{}_{}", base_chat_id, chunk_idx, i);
                    let task = task.clone();
                    tokio::spawn(async move {
                        let msg = IncomingMessage {
                            id: format!("sw_{}_{}", chunk_idx, i),
                            sender_id: "swarm".to_string(),
                            sender_name: None,
                            chat_id,
                            text: task.clone(),
                            is_group: false,
                            reply_to: None,
                            timestamp: chrono::Utc::now(),
                        };
                        let null_ch = NullChannel::new("swarm");
                        let result = runner
                            .handle_message(&msg, &null_ch)
                            .await
                            .unwrap_or_else(|e| format!("⚠️ Agent error: {}", e));
                        (task, result)
                    })
                })
                .collect();

            for handle in handles {
                match handle.await {
                    Ok(result) => all_results.push(result),
                    Err(e) => tracing::warn!("Swarm worker panicked: {}", e),
                }
            }
        }

        all_results
    }

    /// Run the agent loop on a channel.
    pub async fn run(&self, channel: &mut dyn Channel) -> anyhow::Result<()> {
        let mut rx = channel.start().await?;
        tracing::info!("Agent started on channel: {}", channel.name());

        while let Some(msg) = rx.recv().await {
            // Send typing indicator
            let _ = channel.send_typing(&msg.chat_id).await;

            match self.handle_message(&msg, channel).await {
                Ok(response) => {
                    if response.trim().is_empty() {
                        continue;
                    }
                    channel
                        .send(OutgoingMessage {
                            chat_id: msg.chat_id.clone(),
                            text: response,
                            reply_to: Some(msg.id.clone()),
                        })
                        .await?;
                }
                Err(e) => {
                    tracing::error!("Error handling message: {}", e);
                    channel
                        .send(OutgoingMessage {
                            chat_id: msg.chat_id,
                            text: format!("Error: {}", e),
                            reply_to: Some(msg.id),
                        })
                        .await?;
                }
            }
        }

        channel.stop().await?;
        Ok(())
    }

    /// Run the agent loop with an additional message source (e.g., heartbeat).
    pub async fn run_with_extra_rx(
        &self,
        channel: &mut dyn Channel,
        mut extra_rx: mpsc::Receiver<IncomingMessage>,
    ) -> anyhow::Result<()> {
        let mut rx = channel.start().await?;
        tracing::info!(
            "Agent started on channel: {} (with heartbeat)",
            channel.name()
        );

        loop {
            let msg = tokio::select! {
                Some(msg) = rx.recv() => msg,
                Some(msg) = extra_rx.recv() => msg,
                else => break,
            };

            let _ = channel.send_typing(&msg.chat_id).await;

            match self.handle_message(&msg, channel).await {
                Ok(response) => {
                    if msg.sender_id == "system" && response.contains("HEARTBEAT_OK") {
                        tracing::debug!("Heartbeat: agent responded OK, skipping output");
                        continue;
                    }
                    if response.trim().is_empty() {
                        continue;
                    }
                    channel
                        .send(OutgoingMessage {
                            chat_id: msg.chat_id.clone(),
                            text: response,
                            reply_to: Some(msg.id.clone()),
                        })
                        .await?;
                }
                Err(e) => {
                    tracing::error!("Error handling message: {}", e);
                    if msg.sender_id != "system" {
                        channel
                            .send(OutgoingMessage {
                                chat_id: msg.chat_id,
                                text: format!("Error: {}", e),
                                reply_to: Some(msg.id),
                            })
                            .await?;
                    }
                }
            }
        }

        channel.stop().await?;
        Ok(())
    }

    /// Handle a single message — LLM call with tool loop + conversation history.
    pub async fn handle_message(
        &self,
        msg: &IncomingMessage,
        channel: &dyn Channel,
    ) -> anyhow::Result<String> {
        // Send draft placeholder if channel supports live updates, otherwise typing indicator
        let draft_id: Option<String> = if channel.supports_draft_updates() {
            channel.send_draft(&msg.chat_id, "⏳").await.unwrap_or(None)
        } else {
            let _ = channel.send_typing(&msg.chat_id).await;
            None
        };

        if msg.is_group && !crate::context::should_respond(msg) {
            tracing::debug!("Skipping ambient group message without assistant context: {}", msg.id);
            return Ok(String::new());
        }

        // Check for a pending plan awaiting approval (coding mode with plan_approval).
        // If found and approved: resume execution with the original task + stored plan.
        // If rejected: cancel and return early.
        let (effective_text, resume_plan, resume_model) = {
            let mut plans = self.pending_plans.lock().unwrap();
            if let Some(pending) = plans.get(&msg.chat_id) {
                if pending.is_expired() {
                    plans.remove(&msg.chat_id);
                    (msg.text.clone(), None, None)
                } else if is_approval(&msg.text) {
                    let pending = plans.remove(&msg.chat_id).unwrap();
                    tracing::info!("Plan approved for chat_id={}", msg.chat_id);
                    (pending.original_message, Some(pending.plan), Some(pending.preferred_model))
                } else if is_rejection(&msg.text) {
                    plans.remove(&msg.chat_id);
                    return Ok("Plan cancelled. What would you like to do instead?".to_string());
                } else {
                    // New unrelated message — discard the stale pending plan
                    plans.remove(&msg.chat_id);
                    (msg.text.clone(), None, None)
                }
            } else {
                (msg.text.clone(), None, None)
            }
        };
        let resuming_from_plan = resume_plan.is_some();

        // Snapshot mode and hooks once for this message (avoids holding locks across awaits)
        let mode = self.get_mode();
        let hooks_snapshot: Vec<Arc<dyn ToolHook>> = self.hooks.read().unwrap().clone();

        // Build messages: system prompt + conversation history + new message
        let system_prompt = self.system_prompt.read().await.clone();
        let mut messages = vec![ChatMessage::system(&system_prompt)];
        if let Some(guidance) = crate::context::routing_guidance(msg.is_group, channel.name()) {
            messages.push(ChatMessage::system(guidance));
        }

        // Mode-specific system prompt injection
        if let Some(mode_prompt) = mode.system_prompt_injection() {
            messages.push(ChatMessage::system(mode_prompt));
        }

        // Skill injection
        {
            let skills = self.skills.read().await;
            if let Some(skill) = skills::match_skill(&skills, &effective_text) {
                if let Some(content) = skills::load_skill_content(skill) {
                    messages.push(ChatMessage::system(format!(
                        "# Active Skill: {}\n{}\n\nFollow the instructions above for this skill.",
                        skill.name, content
                    )));
                    tracing::info!("Skill matched: {}", skill.name);
                }
            }
        }

        // Load conversation history from the active memory backend
        let history = self
            .memory
            .get_conversation_history(&msg.chat_id, self.agent_config.max_history_messages)
            .await?;
        for (role, content) in history {
            match role.as_str() {
                "user" => messages.push(ChatMessage::user(&content)),
                "assistant" => messages.push(ChatMessage::assistant(&content)),
                _ => {} // Skip unknown roles
            }
        }

        // Add new user message (use original task text when resuming from plan approval)
        messages.push(ChatMessage::user(&effective_text));

        // Tool specs — snapshot at message start
        let tool_specs: Vec<crate::tools::ToolSpec> =
            self.tools.read().await.iter().map(|t| t.spec()).collect();
        let tools_snapshot: Vec<Arc<dyn Tool>> = self.tools.read().await.iter().cloned().collect();
        let main_model = self.model.read().unwrap().clone();

        // ═══════════════════════════════════════════════════════
        // STATE MACHINE: Planning → Executing → Summarizing
        // ═══════════════════════════════════════════════════════

        // Step 1: Decide if this needs planning or is a direct response
        let needs_tools = if resuming_from_plan {
            true // Always execute when resuming an approved plan
        } else {
            self.classify_request(&effective_text, &main_model).await
        };
        let mut state = if needs_tools {
            AgentState::Planning
        } else {
            AgentState::Direct
        };
        tracing::info!("Initial state: {:?}", state);

        // Step 2: If planning, ask Haiku to make a plan + choose execution model
        let mut _plan: Option<String> = None;
        let mut execution_model = main_model.clone(); // Default to configured model (sonnet)

        if let Some(ref plan) = resume_plan {
            // Resuming from an approved plan — inject it and skip re-planning
            messages.push(ChatMessage::system(format!(
                "APPROVED EXECUTION PLAN (follow these steps):\n{}",
                plan
            )));
            execution_model = resume_model.unwrap_or_else(|| main_model.clone());
            state = AgentState::Executing;
            tracing::info!("Resuming with approved plan, model={}", execution_model);
        } else if state == AgentState::Planning {
            let swarm_model_choice = if mode.is_swarm() {
                "   - SWARM: for tasks that can be parallelized across multiple independent agents\n"
            } else {
                ""
            };
            let plan_prompt = format!(
                "You are a planning assistant. Analyze this request and output TWO things:\n\n\
                1. MODEL_CHOICE: Pick ONE execution model:\n\
                   - SONNET: for general tasks, file ops, web, simple edits, queries\n\
                   - OPUS: for complex coding, architecture, multi-file refactors, debugging hard bugs\n\
                   - VIBEMANIA: for building features, creating projects, coding tasks that need autonomous agents\n\
                {swarm_choice}\
                2. PLAN: A brief numbered step-by-step plan.\n\
                   - If VIBEMANIA: the plan should be a single step: delegate to vibemania/subspace with the goal\n\
                   - If SWARM: the plan should list each independent subtask for parallel agents\n\
                   - If SONNET/OPUS: list what tools to use and in what order\n\n\
                Format your response EXACTLY like:\n\
                MODEL_CHOICE: SONNET\n\
                PLAN:\n\
                1. step one\n\
                2. step two\n\n\
                Available tools: {tools}\n\n\
                User request: {request}",
                swarm_choice = swarm_model_choice,
                tools = tool_specs.iter().map(|t| format!("{} ({})", t.name, t.description.chars().take(50).collect::<String>())).collect::<Vec<_>>().join(", "),
                request = &effective_text
            );

            let plan_messages = [ChatMessage::user(&plan_prompt)];
            let plan_request = ChatRequest {
                messages: &plan_messages,
                tools: None,
                model: &self.agent_config.fast_model,
                temperature: 0.8,
                max_tokens: Some(500),
            };

            match self.provider.chat(&plan_request).await {
                Ok(resp) => {
                    let p = resp.text.unwrap_or_default();
                    tracing::info!("Plan: {}", &p[..p.len().min(300)]);

                    // Track cost
                    if let Some(usage) = &resp.usage {
                        let _ = self
                            .cost_tracker
                            .record(
                                &self.agent_config.fast_model,
                                TokenUsage {
                                    input_tokens: usage.input_tokens as usize,
                                    output_tokens: usage.output_tokens as usize,
                                    total_tokens: (usage.input_tokens + usage.output_tokens)
                                        as usize,
                                },
                            )
                            .await;
                    }

                    // Parse model choice from plan
                    let p_upper = p.to_uppercase();
                    if p_upper.contains("MODEL_CHOICE: OPUS")
                        || p_upper.contains("MODEL_CHOICE:OPUS")
                    {
                        execution_model = self.agent_config.heavy_model.clone();
                        tracing::info!("Planner chose OPUS for execution");
                    } else if p_upper.contains("MODEL_CHOICE: SWARM")
                        || p_upper.contains("MODEL_CHOICE:SWARM")
                    {
                        // Route to swarm — inject directive
                        tracing::info!("Planner chose SWARM — routing to coding_swarm tool");
                        messages.push(ChatMessage::system(
                            "IMPORTANT: This task can be parallelized. Use the `coding_swarm` tool \
                            to execute subtasks in parallel agents. \n\
                            Decompose the original goal into a list of independent tasks for the swarm.".to_string()
                        ));
                    } else if p_upper.contains("MODEL_CHOICE: VIBEMANIA")
                        || p_upper.contains("MODEL_CHOICE:VIBEMANIA")
                    {
                        // Route to vibemania — inject directive
                        tracing::info!("Planner chose VIBEMANIA — routing to vibemania tool");
                        messages.push(ChatMessage::system(
                            "IMPORTANT: This is a complex coding task. Use the `vibemania` tool \
                            to handle this autonomously. \n\
                            Do NOT write code yourself — delegate to vibemania.".to_string()
                        ));
                    }
                    // else: stays as main_model (sonnet)

                    // Coding/swarm mode: upgrade to heavy model if planner defaulted to main
                    if mode.prefer_heavy_model() && execution_model == main_model {
                        execution_model = self.agent_config.heavy_model.clone();
                        tracing::info!("Coding mode: upgraded to heavy model");
                    }

                    // Inject plan
                    messages.push(ChatMessage::system(format!(
                        "EXECUTION PLAN (follow these steps):\n{}",
                        p
                    )));
                    _plan = Some(p.clone());
                    state = AgentState::Executing;

                    // Coding mode with plan_approval: return the plan to the user and wait
                    // for explicit confirmation before executing anything.
                    if mode.plan_approval() {
                        {
                            let mut plans = self.pending_plans.lock().unwrap();
                            // Prune stale entries while we have the lock
                            plans.retain(|_, v| !v.is_expired());
                            plans.insert(
                                msg.chat_id.clone(),
                                PendingPlan {
                                    plan: p.clone(),
                                    original_message: effective_text.clone(),
                                    preferred_model: execution_model.clone(),
                                    created_at: std::time::Instant::now(),
                                },
                            );
                        }
                        let plan_response = format!(
                            "**Coding Plan**\n\n{}\n\n---\nReply **go** to execute or **cancel** to abort.",
                            p
                        );
                        if let Some(ref mid) = draft_id {
                            let _ = channel.finalize_draft(&msg.chat_id, mid, &plan_response).await;
                            return Ok(String::new());
                        }
                        return Ok(plan_response);
                    }
                }
                Err(e) => {
                    tracing::warn!("Planning failed ({}), falling back to direct", e);
                    state = AgentState::Direct;
                }
            }
        } // end else if state == AgentState::Planning

        // Step 3: Execute (tool loop)
        let mut tool_call_history: Vec<String> = Vec::new();
        let mut compactions_done: usize = 0;

        for round in 0..self.agent_config.max_rounds {
            // Check for steering messages
            {
                let mut queue = self.steering_queue.lock().unwrap();
                if !queue.is_empty() {
                    for steer_msg in queue.drain(..) {
                        tracing::info!("Steering: {}", &steer_msg[..steer_msg.len().min(80)]);
                        messages.push(ChatMessage::user(format!(
                            "⚡ STEERING — new instruction from user (prioritize this): {}",
                            steer_msg
                        )));
                    }
                }
            }

            // Context budget check — compact if too large
            let context_chars: usize = messages.iter().map(|m| m.content.len()).sum();
            if context_chars > self.agent_config.max_context_chars {
                tracing::info!(
                    "Compacting at round {} ({} chars)",
                    round + 1,
                    context_chars
                );
                messages = self.compact_messages(messages, &effective_text).await?;
                compactions_done += 1;
            }

            // Select model + temperature based on state
            let (model, temperature) = match state {
                AgentState::Planning => (self.agent_config.fast_model.clone(), 0.8),
                AgentState::Executing => (execution_model.clone(), 0.2),
                AgentState::Summarizing => (self.agent_config.fast_model.clone(), 0.7),
                AgentState::Direct => (main_model.clone(), 0.7),
            };

            tracing::info!(
                "[{:?}] round {} — {} msgs, ~{} chars, model={}",
                state,
                round + 1,
                messages.len(),
                messages.iter().map(|m| m.content.len()).sum::<usize>(),
                model
            );

            let request = ChatRequest {
                messages: &messages,
                tools: if tool_specs.is_empty() || state == AgentState::Summarizing {
                    None // No tools during summarization
                } else {
                    Some(&tool_specs)
                },
                model: &model,
                temperature,
                max_tokens: Some(8192),
            };

            let response = self.provider.chat(&request).await?;

            // Track cost
            if let Some(usage) = &response.usage {
                let _ = self
                    .cost_tracker
                    .record(
                        &model,
                        TokenUsage {
                            input_tokens: usage.input_tokens as usize,
                            output_tokens: usage.output_tokens as usize,
                            total_tokens: (usage.input_tokens + usage.output_tokens) as usize,
                        },
                    )
                    .await;
            }

            if !response.has_tool_calls() {
                let text = response.text.unwrap_or_default();

                // State transitions on no tool calls
                match state {
                    AgentState::Executing => {
                        // Execution done — summarize with Haiku if we did significant work
                        if round >= 3 {
                            tracing::info!(
                                "Execution done after {} rounds, summarizing",
                                round + 1
                            );
                            state = AgentState::Summarizing;
                            messages.push(ChatMessage::assistant(text));
                            messages.push(ChatMessage::user(
                                "Now provide a clean, concise final response to the user. \
                                Summarize what you did and the results. Be brief and direct."
                                    .to_string(),
                            ));
                            continue; // One more round with Haiku for summary
                        }
                        // Short execution — return directly
                        tracing::info!("Done after {} round(s) [{:?}]", round + 1, state);
                        self.persist_conversation(msg, &text).await?;
                        if let Some(ref mid) = draft_id {
                            let _ = channel.finalize_draft(&msg.chat_id, mid, &text).await;
                            return Ok(String::new()); // channel sent it
                        }
                        return Ok(text);
                    }
                    AgentState::Summarizing | AgentState::Direct | AgentState::Planning => {
                        tracing::info!("Done after {} round(s) [{:?}]", round + 1, state);
                        self.persist_conversation(msg, &text).await?;
                        if let Some(ref mid) = draft_id {
                            let _ = channel.finalize_draft(&msg.chat_id, mid, &text).await;
                            return Ok(String::new()); // channel sent it
                        }
                        return Ok(text);
                    }
                }
            }

            // === TOOL EXECUTION (only in Executing or Direct state) ===

            // Loop detection
            for tc in &response.tool_calls {
                let hash = format!(
                    "{}:{}",
                    tc.name,
                    &tc.arguments[..tc.arguments.len().min(200)]
                );
                tool_call_history.push(hash);
            }
            if tool_call_history.len() >= LOOP_BREAK_THRESHOLD {
                let last = &tool_call_history[tool_call_history.len() - 1];
                let consecutive = tool_call_history
                    .iter()
                    .rev()
                    .take_while(|h| *h == last)
                    .count();
                if consecutive >= LOOP_BREAK_THRESHOLD {
                    tracing::warn!("Loop: {} identical calls, breaking", consecutive);
                    return Ok(format!(
                        "Loop detected ({} identical {} calls). Stopping.",
                        consecutive, response.tool_calls[0].name
                    ));
                }
                if consecutive >= LOOP_WARN_THRESHOLD {
                    messages.push(ChatMessage::user(
                        "WARNING: You're repeating tool calls. Stop and answer with what you have."
                            .to_string(),
                    ));
                }
            }

            // Progress callback
            if !channel.supports_draft_updates() {
                let _ = channel.send_typing(&msg.chat_id).await;
            }

            // Build assistant tool_use message
            {
                let mut content_blocks: Vec<serde_json::Value> = Vec::new();
                if let Some(text) = &response.text {
                    if !text.is_empty() {
                        content_blocks.push(serde_json::json!({
                            "type": "text", "text": text,
                        }));
                    }
                }
                for tc in &response.tool_calls {
                    content_blocks.push(serde_json::json!({
                        "type": "tool_use",
                        "id": &tc.id,
                        "name": &tc.name,
                        "input": serde_json::from_str::<serde_json::Value>(&tc.arguments).unwrap_or_default(),
                    }));
                }
                messages.push(ChatMessage {
                    role: "assistant_tool_use".to_string(),
                    content: String::new(),
                    tool_use_id: Some(serde_json::to_string(&content_blocks).unwrap_or_default()),
                });
            }

            // Execute tools
            tracing::info!(
                "Tools: {}",
                response
                    .tool_calls
                    .iter()
                    .map(|tc| tc.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            // Build accumulated progress text for draft updates
            let mut progress_lines: Vec<String> = Vec::new();

            for tc in &response.tool_calls {
                // ── Progress: tool start ──────────────────────────────
                if let (true, Some(ref mid)) = (channel.supports_draft_updates(), &draft_id) {
                    let hint = extract_tool_hint(&tc.name, &tc.arguments);
                    let start_line = if hint.is_empty() {
                        format!("⏳ {}\n", tc.name)
                    } else {
                        format!("⏳ {}: {}\n", tc.name, hint)
                    };
                    progress_lines.push(start_line.clone());
                    let _ = channel
                        .update_draft_progress(&msg.chat_id, mid, &progress_lines.join(""))
                        .await;
                }

                let started = std::time::Instant::now();

                // Pre-tool hooks — a Block decision skips execution entirely
                let result = match run_pre_hooks(&hooks_snapshot, &tc.name, &tc.arguments).await {
                    HookDecision::Block(reason) => {
                        tracing::info!("Hook blocked '{}': {}", tc.name, reason);
                        crate::tools::ToolResult::error(format!("Blocked by policy: {}", reason))
                    }
                    HookDecision::Allow => {
                        if let Some(tool) = tools_snapshot.iter().find(|t| t.name() == tc.name) {
                            match tool.execute(&tc.arguments).await {
                                Ok(r) => r,
                                Err(e) => {
                                    crate::tools::ToolResult::error(format!("Tool error: {}", e))
                                }
                            }
                        } else {
                            crate::tools::ToolResult::error(format!(
                                "Unknown tool: {}",
                                tc.name
                            ))
                        }
                    }
                };

                // Post-tool hooks (logging, auditing, etc.)
                run_post_hooks(&hooks_snapshot, &tc.name, &tc.arguments, &result).await;

                let elapsed = started.elapsed().as_secs();

                // ── Progress: tool completion ─────────────────────────
                if let (true, Some(ref mid)) = (channel.supports_draft_updates(), &draft_id) {
                    // Replace the ⏳ line for this tool with a ✅/❌ line
                    let done_line = if result.is_error {
                        format!("❌ {} ({}s)\n", tc.name, elapsed)
                    } else {
                        format!("✅ {} ({}s)\n", tc.name, elapsed)
                    };
                    // Replace last line that started with "⏳ {name}"
                    let prefix = format!("⏳ {}", tc.name);
                    if let Some(pos) = progress_lines.iter().rposition(|l| l.starts_with(&prefix)) {
                        progress_lines[pos] = done_line;
                    } else {
                        progress_lines.push(done_line);
                    }
                    let _ = channel
                        .update_draft_progress(&msg.chat_id, mid, &progress_lines.join(""))
                        .await;
                }

                let truncated_output =
                    if result.output.len() > self.agent_config.max_tool_result_chars {
                        format!(
                            "{}...\n⚠️ [Truncated {} → {} chars]",
                            &result.output[..self.agent_config.max_tool_result_chars],
                            result.output.len(),
                            self.agent_config.max_tool_result_chars
                        )
                    } else {
                        result.output.clone()
                    };

                messages.push(ChatMessage::tool_result(&tc.id, &truncated_output));
            }

            // After first tool response in Direct mode, switch to Executing
            if state == AgentState::Direct {
                state = AgentState::Executing;
            }
        }

        tracing::warn!(
            "Circuit breaker after {} rounds",
            self.agent_config.max_rounds
        );
        self.persist_conversation(
            msg,
            &format!("Hit {} rounds.", self.agent_config.max_rounds),
        )
        .await?;
        let circuit_msg = format!(
            "⚠️ Hit {} rounds ({} compactions). Break into smaller tasks?",
            self.agent_config.max_rounds, compactions_done
        );
        if let Some(ref mid) = draft_id {
            let _ = channel.finalize_draft(&msg.chat_id, mid, &circuit_msg).await;
            return Ok(String::new());
        }
        Ok(circuit_msg)
    }

    /// Classify if a request needs tool calls (and thus planning) or is conversational
    async fn classify_request(&self, text: &str, _model: &str) -> bool {
        // BypassPermissions, Coding, and Swarm modes always plan
        if self.get_mode().always_plan() {
            return true;
        }

        // Heuristic: if message is short and conversational, skip planning
        let lower = text.to_lowercase();
        let word_count = text.split_whitespace().count();

        // Short messages are usually conversational
        if word_count <= 5 {
            return false;
        }

        // Explicit tool-needing keywords
        let tool_keywords = [
            "read ",
            "write ",
            "edit ",
            "create ",
            "build ",
            "fix ",
            "search ",
            "fetch ",
            "check ",
            "run ",
            "execute ",
            "install ",
            "deploy ",
            "find ",
            "list ",
            "show me ",
            "what's in ",
            "look at ",
            "file",
            "code",
            "commit",
            "git ",
            "grep",
            "curl",
        ];
        for kw in &tool_keywords {
            if lower.contains(kw) {
                return true;
            }
        }

        // Longer messages with questions are likely complex tasks
        if word_count >= 15
            && (lower.contains('?') || lower.contains("can you") || lower.contains("please"))
        {
            return true;
        }

        false
    }

    /// Persist user + assistant messages to conversation history
    async fn persist_conversation(
        &self,
        msg: &IncomingMessage,
        response: &str,
    ) -> anyhow::Result<()> {
        self.memory
            .store_conversation_batch(&[
                (&msg.chat_id, &msg.sender_id, "user", &msg.text),
                (&msg.chat_id, "assistant", "assistant", response),
            ])
            .await?;
        Ok(())
    }

    /// Compact conversation using Haiku — summarize old messages, keep recent ones
    async fn compact_messages(
        &self,
        messages: Vec<ChatMessage>,
        original_task: &str,
    ) -> anyhow::Result<Vec<ChatMessage>> {
        // Keep: system prompt + last 6 messages (3 exchanges)
        let keep_recent = 6;

        if messages.len() <= keep_recent + 2 {
            return Ok(messages); // Nothing to compact
        }

        // Split: system messages + old messages + recent messages
        let system_msgs: Vec<&ChatMessage> =
            messages.iter().filter(|m| m.role == "system").collect();
        let non_system: Vec<&ChatMessage> =
            messages.iter().filter(|m| m.role != "system").collect();

        if non_system.len() <= keep_recent {
            return Ok(messages);
        }

        let (old_msgs, recent_msgs) = non_system.split_at(non_system.len() - keep_recent);

        // Build summary of old messages for Haiku
        let mut summary_input = String::new();
        for m in old_msgs {
            let role_label = match m.role.as_str() {
                "user" => "User",
                "assistant" | "assistant_tool_use" => "Assistant",
                "tool_result" => "Tool Result",
                _ => &m.role,
            };
            // Truncate each message for the summary request
            let content = if m.content.len() > 500 {
                format!("{}...", &m.content[..500])
            } else {
                m.content.clone()
            };
            summary_input.push_str(&format!("[{}]: {}\n", role_label, content));
        }

        // Ask Haiku to summarize
        let compaction_prompt = format!(
            "Summarize this conversation concisely. The original task was: \"{}\"\n\n\
            Focus on: what was accomplished, what tools were used, key results, and what's still pending.\n\n\
            Conversation:\n{}",
            original_task,
            summary_input
        );

        let compact_messages = [ChatMessage::user(&compaction_prompt)];
        let compact_request = ChatRequest {
            messages: &compact_messages,
            tools: None,
            model: &self.agent_config.fast_model,
            temperature: 0.3,
            max_tokens: Some(1000),
        };

        let summary = match self.provider.chat(&compact_request).await {
            Ok(resp) => {
                // Track cost for compaction
                if let Some(usage) = &resp.usage {
                    let _ = self
                        .cost_tracker
                        .record(
                            &self.agent_config.fast_model,
                            TokenUsage {
                                input_tokens: usage.input_tokens as usize,
                                output_tokens: usage.output_tokens as usize,
                                total_tokens: (usage.input_tokens + usage.output_tokens) as usize,
                            },
                        )
                        .await;
                }
                resp.text
                    .unwrap_or_else(|| "Failed to summarize.".to_string())
            }
            Err(e) => {
                tracing::warn!("Compaction failed: {}, falling back to truncation", e);
                // Fallback: just truncate old messages
                format!(
                    "[Previous {} messages truncated to save context]",
                    old_msgs.len()
                )
            }
        };

        // Rebuild messages: system + summary + recent
        let mut compacted = Vec::new();
        for sm in &system_msgs {
            compacted.push((*sm).clone());
        }
        compacted.push(ChatMessage::user(format!(
            "[Conversation compacted — {} earlier messages summarized]\n\n{}",
            old_msgs.len(),
            summary
        )));
        compacted.push(ChatMessage::assistant(
            "Understood, continuing from the summary.".to_string(),
        ));
        for rm in recent_msgs {
            compacted.push((*rm).clone());
        }

        Ok(compacted)
    }
}

/// Extract a short hint from tool arguments for progress display.
/// e.g. shell → first 60 chars of "command"; web_search → "query"; file_ops → "path"
fn extract_tool_hint(name: &str, arguments: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(arguments).unwrap_or_default();
    let hint = match name {
        "shell" | "bash" | "exec" => v
            .get("command")
            .or_else(|| v.get("cmd"))
            .and_then(|s| s.as_str()),
        "web_search" | "search" => v
            .get("query")
            .or_else(|| v.get("q"))
            .and_then(|s| s.as_str()),
        "web_fetch" | "fetch" => v.get("url").and_then(|s| s.as_str()),
        "file_ops" | "read" | "write" | "edit" => v
            .get("path")
            .or_else(|| v.get("file_path"))
            .and_then(|s| s.as_str()),
        "vibemania" => v.get("goal").and_then(|s| s.as_str()),
        _ => v
            .as_object()
            .and_then(|o| o.values().next())
            .and_then(|v| v.as_str()),
    };
    hint.map(|s| {
        let s = s.trim();
        if s.len() > 60 {
            format!("{}…", &s[..57])
        } else {
            s.to_string()
        }
    })
    .unwrap_or_default()
}
