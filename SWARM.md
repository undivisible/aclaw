# SWARM.md — Multi-Agent Swarm System

## Overview

unthinkclaw supports multi-agent coordination via the `swarm` feature flag. This enables:

- **Agent Delegation** — Named agents delegate tasks to other named agents with permission controls
- **Agent Teams** — Shared task boards with atomic claiming, dependency tracking, and mailboxes
- **Agent Handoff** — Transfer conversation control between agents with routing overrides
- **Evaluate Loop** — Generator-evaluator feedback cycle with quality gates
- **Concurrency Management** — Lane-based scheduling with deadlock detection

## Architecture

```
src/swarm/
  mod.rs              — Module exports + feature gates
  models.rs           — Data models (AgentLink, Team, TeamTask, etc.)
  storage.rs          — SwarmStorage trait + SurrealDB backend + RocksDB cache
  coordinator.rs      — Central orchestrator (owns all managers)
  delegation.rs       — Inter-agent delegation with permissions + concurrency limits
  team.rs             — Team lifecycle, task board, mailbox
  handoff.rs          — Conversation routing override
  evaluate.rs         — Generator-evaluator feedback loop
  scheduler.rs        — Lane-based concurrency scheduler + deadlock detection
  task_queue.rs       — Basic task model (legacy)
  agent_registry.rs   — Agent identity, capabilities, model config
```

## Building

```bash
# Without swarm (default — single agent)
cargo build --release

# With swarm
cargo build --release --features swarm
```

Swarm requires SurrealDB (embedded RocksDB engine) and RocksDB.

## Storage

### SurrealDB Tables

| Table | Purpose |
|-------|---------|
| `agents` | Agent registry (name, model, tools, capabilities, status) |
| `agent_links` | Delegation permissions between agents |
| `delegation_history` | Tracks all delegation requests and outcomes |
| `teams` | Team definitions with lead agent |
| `team_members` | Team membership with roles |
| `team_tasks` | Task board with dependencies and atomic claiming |
| `team_messages` | Team mailbox for peer messaging |
| `handoff_routes` | Conversation routing overrides |
| `tasks` | Legacy task queue (backward compat) |

### RocksDB Cache

Column families: `embeddings`, `chunks`, `sticker_cache`, `agent_cache`

Used for local hot data that doesn't need distributed access.

## Agent Delegation

### Concepts

- **Agent Link**: Permission for one agent to delegate to another
- **Direction**: `outbound` (A→B), `inbound` (B→A), `bidirectional` (A↔B)
- **Concurrency Limits**: Per-link and per-agent maximums
- **Modes**: `sync` (wait for result) or `async` (fire and forget)

### CLI

```bash
# Register agents
unthinkclaw swarm agent-create coder --model claude-sonnet-4-5 --capabilities coding,testing
unthinkclaw swarm agent-create researcher --model claude-sonnet-4-5 --capabilities research

# Create delegation link
unthinkclaw swarm agent-link coder researcher --direction outbound --max-concurrent 3

# List agents
unthinkclaw swarm agents
```

### Flow

1. Source agent calls `delegate(target_name, task, mode)`
2. System checks `agent_links` for permission
3. System checks per-link and per-agent concurrency limits
4. Delegation record created in `delegation_history`
5. Target agent executes task
6. Result returned (sync) or announced (async)

## Teams

### Concepts

- **Team**: Group of agents with a lead
- **Task Board**: Prioritized tasks with dependency tracking
- **Atomic Claiming**: SurrealDB transactions prevent double-assignment
- **Blocked Tasks**: Tasks with `blocked_by` dependencies wait automatically
- **Mailbox**: Broadcast or directed messages between team members

### CLI

```bash
# Create team
unthinkclaw swarm team-create security-audit --lead coder

# Add task
unthinkclaw swarm team-task-add security-audit "Review auth module" --priority 5

# Add task with dependency
unthinkclaw swarm team-task-add security-audit "Write fix" --blocked-by <task-id>

# List teams
unthinkclaw swarm teams
```

### Task States

```
pending → claimed → done
    ↓         ↓
  blocked   failed
```

- `pending`: Ready to be claimed
- `blocked`: Waiting on `blocked_by` tasks to complete
- `claimed`: Assigned to an agent (atomic)
- `done`: Completed with result
- `failed`: Failed with error

When a blocking task completes, blocked tasks automatically move to `pending`.

## Handoff

Transfer conversation control from one agent to another:

```
User → Agent A → (handoff) → Agent B
                              ↑
                    Future messages routed here
```

The handoff route is stored in `handoff_routes` table. When a message arrives for a channel+chat_id with an active route, it goes to the target agent instead of the default.

## Evaluate Loop

Generator-evaluator feedback cycle:

1. Generator produces output from prompt
2. Evaluator scores output (0.0-1.0) and provides feedback
3. If score < threshold, generator revises
4. Max 5 revision rounds (configurable, max 10)
5. Returns best result

```rust
let config = EvaluateConfig {
    generator_prompt: "Write a security audit report...".into(),
    evaluator_prompt: "Evaluate this report for completeness...".into(),
    quality_threshold: 0.8,
    max_rounds: 5,
    ..Default::default()
};

let result = evaluate_loop(provider, &config).await?;
println!("Score: {:.0}%, Rounds: {}", result.final_score * 100.0, result.rounds.len());
```

## Concurrency Scheduler

### Lanes

| Lane | Default Max | Priority | Purpose |
|------|-------------|----------|---------|
| Main | 3 | 2 (highest) | Primary agent interactions |
| Delegate | 5 | 1 | Delegated tasks |
| Cron | 2 | 0 (lowest) | Scheduled background tasks |

### Deadlock Detection

The scheduler maintains a wait graph tracking which agents are waiting on which other agents. Cycle detection runs on status queries and reports circular dependencies.

### CLI

```bash
unthinkclaw swarm status
```

## All CLI Commands

```
unthinkclaw swarm start               # Initialize coordinator
unthinkclaw swarm agent-create <name> # Register agent
unthinkclaw swarm agent-link <a> <b>  # Create delegation permission
unthinkclaw swarm team-create <name>  # Create team
unthinkclaw swarm team-task-add <t>   # Add task to team board
unthinkclaw swarm agents              # List agents
unthinkclaw swarm tasks               # List pending tasks
unthinkclaw swarm teams               # List teams
unthinkclaw swarm delegations <agent> # List active delegations
unthinkclaw swarm task <desc>         # Submit legacy task
unthinkclaw swarm queue <msg>         # Queue steering message
unthinkclaw swarm status              # Scheduler status
```

## Design Decisions

1. **SurrealDB embedded (RocksDB engine)** — No external database required, but supports distributed mode if needed
2. **Trait-based storage** — `SwarmStorage` trait allows alternative backends
3. **In-memory concurrency tracking** — Fast path for limit checks, synced from storage on startup
4. **Atomic task claiming** — SurrealDB transactions prevent race conditions
5. **Feature-gated** — Zero overhead when not using swarm (`--features swarm`)
6. **Backward compatible** — Existing single-agent functionality unchanged
