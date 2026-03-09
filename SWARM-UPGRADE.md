# unthinkclaw Multi-Agent Swarm Upgrade

## Goal
Transform unthinkclaw from in-memory swarm to distributed multi-agent system with SurrealDB (shared state) + RocksDB (local cache).

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  unthinkclaw Swarm Network                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ Agent 1  │  │ Agent 2  │  │ Agent 3  │  │ Agent N  │  │
│  │ (Rust)   │  │ (Rust)   │  │ (Rust)   │  │ (Rust)   │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  │
│       │             │             │             │          │
│       └─────────────┼─────────────┼─────────────┘          │
│                     │             │                        │
│          ┌──────────▼─────────────▼──────────┐            │
│          │      SurrealDB (Shared State)     │            │
│          │  - tasks (pending/running/done)   │            │
│          │  - agents (id, status, heartbeat) │            │
│          │  - conversations (chat history)   │            │
│          │  - context (key-value store)      │            │
│          │  - events (pub/sub bus)           │            │
│          └──────────┬─────────────┬──────────┘            │
│                     │             │                        │
│          ┌──────────▼─────────────▼──────────┐            │
│          │      RocksDB (Local Cache)        │            │
│          │  - embeddings (vector cache)      │            │
│          │  - chunks (file content)          │            │
│          │  - stickers (vision analysis)     │            │
│          └───────────────────────────────────┘            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Phase 1: Backend Abstraction Layer

### 1.1 Create `MemoryBackend` Trait
```rust
// src/memory/backend.rs
#[async_trait]
pub trait MemoryBackend: Send + Sync {
    // Conversations
    async fn store_conversation(&self, chat_id: &str, messages: &[Message]) -> Result<()>;
    async fn get_conversation(&self, chat_id: &str, limit: usize) -> Result<Vec<Message>>;
    
    // Context store (key-value)
    async fn context_set(&self, namespace: &str, key: &str, value: &Value, ttl: Option<Duration>) -> Result<()>;
    async fn context_get(&self, namespace: &str, key: &str) -> Result<Option<Value>>;
    async fn context_delete(&self, namespace: &str, key: &str) -> Result<()>;
    async fn context_list(&self, namespace: &str) -> Result<Vec<String>>;
    
    // Events (pub/sub)
    async fn publish_event(&self, target: &str, event_type: &str, data: &Value) -> Result<()>;
    async fn subscribe_events(&self, target: &str) -> Result<EventStream>;
    
    // Search
    async fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
}
```

### 1.2 Implement Backends
- `SqliteBackend` (existing, single-agent mode)
- `SurrealBackend` (distributed swarm)
- `HybridBackend` (Surreal + RocksDB cache)

## Phase 2: SurrealDB Integration

### 2.1 Schema Definition
```rust
// src/memory/surreal.rs
pub struct SurrealBackend {
    db: Surreal<Client>,
    namespace: String,
    database: String,
}

// Tables:
// - tasks: { id, goal, priority, status, assigned_to, created_at, updated_at }
// - agents: { id, model, status, last_heartbeat, capabilities, current_task }
// - conversations: { chat_id, messages, updated_at }
// - context: { namespace, key, value, ttl, created_at }
// - events: { target, event_type, data, priority, created_at }
```

### 2.2 Task Queue Logic
```rust
impl SurrealBackend {
    // Atomic task claim (prevent double-assignment)
    async fn claim_task(&self, agent_id: &str) -> Result<Option<Task>> {
        // SurrealDB transaction:
        // 1. SELECT * FROM tasks WHERE status = 'pending' ORDER BY priority DESC LIMIT 1
        // 2. UPDATE task SET status = 'assigned', assigned_to = agent_id
        // 3. RETURN task
    }
    
    // Leader election via LIVE queries
    async fn elect_leader(&self) -> Result<String> {
        // Use SurrealDB LIVE queries to watch agent heartbeats
        // Agent with lowest ID becomes leader
    }
}
```

### 2.3 Connection String
```bash
unthinkclaw swarm --surreal ws://localhost:8000 --namespace claw --database swarm
```

## Phase 3: RocksDB Local Cache

### 3.1 Cache Strategy
```rust
// src/memory/rocksdb.rs
pub struct RocksCache {
    db: rocksdb::DB,
}

impl RocksCache {
    // Column families
    const CF_EMBEDDINGS: &str = "embeddings";
    const CF_CHUNKS: &str = "chunks";
    const CF_STICKERS: &str = "stickers";
    
    // Write-through: always write to SurrealDB first, then cache
    async fn cache_embedding(&self, key: &str, vector: &[f32]) -> Result<()>;
    
    // Read-through: check cache first, fallback to SurrealDB
    async fn get_embedding(&self, key: &str) -> Result<Option<Vec<f32>>>;
}
```

### 3.2 Hybrid Backend
```rust
pub struct HybridBackend {
    surreal: SurrealBackend,
    rocks: RocksCache,
}

impl MemoryBackend for HybridBackend {
    async fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // 1. Check RocksDB cache
        if let Some(results) = self.rocks.get_cached_search(query, limit).await? {
            return Ok(results);
        }
        
        // 2. Query SurrealDB
        let results = self.surreal.fts_search(query, limit).await?;
        
        // 3. Cache results
        self.rocks.cache_search(query, &results).await?;
        
        Ok(results)
    }
}
```

## Phase 4: Swarm Coordinator

### 4.1 Agent Registration
```rust
// src/swarm/coordinator.rs
pub struct SwarmCoordinator {
    backend: Arc<dyn MemoryBackend>,
    agent_id: String,
    heartbeat_interval: Duration,
}

impl SwarmCoordinator {
    pub async fn register(&self) -> Result<()> {
        // Insert into agents table
        // Start heartbeat loop
    }
    
    async fn heartbeat_loop(&self) {
        loop {
            self.backend.update_heartbeat(&self.agent_id).await;
            tokio::time::sleep(self.heartbeat_interval).await;
        }
    }
    
    pub async fn next_task(&self) -> Result<Option<Task>> {
        self.backend.claim_task(&self.agent_id).await
    }
}
```

### 4.2 Task Distribution
```rust
impl SwarmCoordinator {
    // Round-robin with capability matching
    pub async fn assign_tasks(&self) -> Result<()> {
        let agents = self.backend.list_agents().await?;
        let tasks = self.backend.list_pending_tasks().await?;
        
        for task in tasks {
            // Find agent with matching capabilities
            if let Some(agent) = self.find_capable_agent(&agents, &task) {
                self.backend.assign_task(&task.id, &agent.id).await?;
            }
        }
        
        Ok(())
    }
}
```

### 4.3 Leader Election
```rust
impl SwarmCoordinator {
    pub async fn run_leader_duties(&self) -> Result<()> {
        loop {
            // Check for dead agents (heartbeat > 30s ago)
            self.mark_dead_agents().await?;
            
            // Reassign orphaned tasks
            self.reassign_orphaned_tasks().await?;
            
            // Distribute pending tasks
            self.assign_tasks().await?;
            
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }
}
```

## Phase 5: CLI Commands

### 5.1 Swarm Management
```bash
# Start swarm node (joins existing or creates new)
unthinkclaw swarm start --surreal ws://localhost:8000 --namespace claw --database swarm

# Spawn N agents
unthinkclaw swarm spawn --count 5 --model claude-opus-4

# List active agents
unthinkclaw swarm agents

# Submit task to swarm
unthinkclaw swarm task "refactor auth module" --priority 8

# Get task status
unthinkclaw swarm status <task-id>

# Kill agent
unthinkclaw swarm kill <agent-id>

# Swarm health
unthinkclaw swarm health
```

### 5.2 Context Store
```bash
# Set value
unthinkclaw context set --namespace proj-a --key spec --value '{"goal":"ship"}'

# Get value
unthinkclaw context get --namespace proj-a --key spec

# List keys
unthinkclaw context list --namespace proj-a

# Publish event
unthinkclaw event publish --target orchestrator --type task_complete --data '{"result":"ok"}'
```

## Phase 6: Workflow Patterns

### 6.1 Workflow Types
```rust
#[async_trait]
pub trait Workflow: Send + Sync {
    fn workflow_type(&self) -> &str;
    async fn execute(&self, ctx: &WorkflowContext) -> Result<WorkflowResult>;
}

pub struct ConcurrentWorkflow; // Parallel execution
pub struct PipelineWorkflow;   // Sequential chaining
pub struct IterativeWorkflow;  // Loop until success
```

### 6.2 Auto-Detection
```rust
pub struct WorkflowClassifier;

impl WorkflowClassifier {
    pub fn classify(task: &str) -> WorkflowType {
        let task_lower = task.to_lowercase();
        
        if task_lower.contains("search") || task_lower.contains("research") {
            WorkflowType::Concurrent
        } else if task_lower.contains("refactor") || task_lower.contains("pipeline") {
            WorkflowType::Pipeline
        } else if task_lower.contains("retry") || task_lower.contains("until") {
            WorkflowType::Iterative
        } else {
            WorkflowType::Concurrent // default
        }
    }
}
```

## Phase 7: Production Hardening

### 7.1 Error Recovery
- Automatic agent respawn on crash
- Task retry with exponential backoff
- Orphaned task reassignment
- Corrupted state recovery (backup + restore)

### 7.2 Monitoring
- Prometheus metrics export
- Task execution time tracking
- Agent health checks
- Dead letter queue for failed tasks

### 7.3 Security
- Agent authentication (JWT tokens)
- Namespace isolation
- Rate limiting per agent
- Resource quotas (max tasks, memory)

## Implementation Order

1. ✅ **Phase 1**: Backend abstraction (trait + SQLite impl)
2. 🚀 **Phase 2**: SurrealDB integration
3. 🚀 **Phase 3**: RocksDB cache layer
4. 🚀 **Phase 4**: Swarm coordinator
5. 🚀 **Phase 5**: CLI commands
6. 🚀 **Phase 6**: Workflow patterns
7. 🚀 **Phase 7**: Production hardening

## Dependencies to Add

```toml
[dependencies]
surrealdb = "2.2"
rocksdb = "0.22"
async-trait = "0.1"
tokio = { version = "1", features = ["full"] }
uuid = { version = "1", features = ["v4"] }
chrono = "0.4"
```

## Testing Strategy

1. **Unit tests**: Each backend implementation
2. **Integration tests**: Swarm coordinator with mock backends
3. **Stress tests**: 100+ agents, 1000+ tasks
4. **Failure tests**: Network partitions, agent crashes, DB failures

## Rollout Plan

1. Keep SQLite as default (backward compat)
2. Add `--backend` flag: `sqlite` (default), `surreal`, `hybrid`
3. Document migration path from SQLite → SurrealDB
4. Provide docker-compose for SurrealDB + RocksDB setup

---

**Ready to start Phase 2?** Say the word and I'll implement SurrealDB backend. 🐾
