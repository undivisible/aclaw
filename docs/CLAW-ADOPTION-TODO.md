# Claw Adoption Todo

Purpose: turn the cross-repo review into an execution backlog for `unthinkclaw`.

Verified snapshot on 2026-03-18:
- validation passes:
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
  - `cargo build --release`
- GitHub issues `#2` through `#6` are resolved and closed
- Hermes-style foundations landed:
  - toolset filtering
  - session search primitives
  - managed skill persistence
  - Daytona runtime scaffold

Status key:
- `[ ]` not started
- `[-]` in progress
- `[x]` done
- `[!]` blocked / needs decision

## North Star

- [ ] Keep `unthinkclaw` lean and Rust-first while borrowing the strongest ideas from ZeroClaw, OpenClaw, GoClaw, NanoClaw, MicroClaw, and HiClaw.
- [ ] Preserve full-computer-agent behavior:
  - unrestricted file access
  - unrestricted website access
  - gateway treated as an authenticated control plane, not a public admin surface
- [ ] Move persistent storage toward SurrealDB + RocksDB as the primary long-term state layer for memory, swarm, and coordination data.

## Already Landed

- [x] Policy-gated privileged execution for shell, dynamic tools, and plugins
- [x] Gateway now defaults to localhost and requires auth
- [x] Agent loop avoids repeated full context cloning
- [x] SQLite memory now has pooled concurrency and FTS-first search
- [x] SurrealDB + RocksDB memory backend exists and is preferred in swarm builds
- [x] Prompt loading moved to async
- [x] Toolset-based tool exposure filtering exists
- [x] Session search primitives exist across current memory backends
- [x] Managed skill persistence exists
- [x] Daytona runtime scaffold exists

## ZeroClaw Adoption

Focus: security policy model, observability, backend abstraction, stronger tests.

- [ ] Replace coarse execution-policy booleans with a richer `SecurityPolicy` model:
  - capability classes
  - approval requirements
  - channel/session-aware policy decisions
  - explicit dangerous-tool classification
- [ ] Add observer hooks around the agent loop:
  - round start/end
  - tool start/end
  - compaction
  - provider request/response
  - cancellation / abort
- [ ] Add Prometheus-compatible metrics surface.
- [ ] Add structured tracing/OTel hooks behind a lightweight feature flag.
- [ ] Expand memory abstraction so backends support:
  - richer scored search results
  - session-scoped entries
  - backend capability detection
  - future reranking / vector search hooks
- [ ] Add robustness tests modeled after ZeroClaw:
  - agent loop edge cases
  - gateway auth regressions
  - channel webhook security
  - tool-policy enforcement

## OpenClaw Adoption

Focus: gateway discipline, security audits, session routing, operator UX.

- [ ] Add a `security audit` command for config + runtime posture:
  - gateway bind/auth checks
  - risky policy flags
  - remote exposure warnings
  - dangerous tool exposure checks
- [ ] Add explicit dangerous-tool registries:
  - tools denied over gateway HTTP/API by default
  - tools requiring explicit approval
  - tools safe for automation surfaces
- [ ] Split gateway capabilities into surfaces:
  - channel ingress
  - agent control plane
  - operator/admin endpoints
- [ ] Restrict gateway tool invocation to a narrow allowlist instead of exposing full tool execution.
- [ ] Add origin/trusted-proxy/rate-limit checks for any non-loopback deployment mode.
- [ ] Add better session routing primitives:
  - stable session keys
  - main vs subagent/orchestrator distinction
  - explicit parent/child session relationships
- [ ] Add a `doctor`-style config validation command.

## GoClaw Adoption

Focus: serious swarm orchestration, quality gates, observability, admin model.

- [ ] Add swarm task-board primitives:
  - create task
  - claim task
  - complete task
  - blocked-by dependency tracking
  - atomic claim semantics
- [ ] Add swarm mailbox primitives:
  - direct agent-to-agent message
  - broadcast
  - unread/read state
- [ ] Add delegation links / permissions:
  - source agent
  - target agent
  - direction
  - per-link concurrency
  - per-agent delegation load limits
- [ ] Add quality gates / hook engine:
  - command evaluators
  - agent evaluators
  - retry policy
  - recursion prevention
- [ ] Add hybrid delegation target search for large swarms.
- [ ] Add a minimal admin/API surface for:
  - active agents
  - queue depth
  - task board
  - mailbox
  - memory health
- [ ] Add stronger usage analytics and cost/latency metrics.

## NanoClaw Adoption

Focus: simplicity, isolation boundaries, queue discipline.

- [ ] Clarify queue ownership model:
  - per-chat queue
  - per-agent queue
  - global concurrency cap
  - fair scheduling rules
- [ ] Simplify orchestration code paths so the hot path is easier to audit.
- [ ] Add optional isolated execution mode for swarm workers:
  - containerized worker runtime
  - explicit mounts
  - restricted environment
  - keep host-mode full-computer-agent support as the default/optional policy choice
- [ ] Reduce config sprawl where code-level defaults are sufficient.
- [ ] Document the runtime trust model in one short file.

## MicroClaw Adoption

Focus: Rust runtime ergonomics, session persistence, compaction, task planning.

- [ ] Upgrade compaction flow:
  - archive old conversations before compaction
  - summarize old context
  - keep recent turns verbatim
  - validate thresholds so compaction cannot be configured into a no-op
- [ ] Persist richer session state:
  - tool interactions
  - pending subagent runs
  - compaction metadata
  - last active session bindings
- [ ] Add persistent todo/task tools for the agent runtime:
  - read plan
  - write plan
  - keep it cheap and visible to the user
- [ ] Improve web/setup/doctor ergonomics without bloating the binary.
- [ ] Evaluate whether optional vector search should remain a fallback once Surreal memory is primary.

## HiClaw Adoption

Focus: manager/worker topology, human-visible swarm operation, centralized credentials.

- [ ] Design manager/worker swarm mode:
  - one manager agent
  - many worker agents
  - explicit assignment flow
  - heartbeat / stuck-worker detection
- [ ] Add human-visible swarm state:
  - current workers
  - task assignments
  - last heartbeat
  - failure reason
- [ ] Centralize external credentials at the control-plane boundary where practical.
- [ ] Evaluate Matrix or similar room-based swarm collaboration as an optional deployment mode.
- [ ] Add shared artifact/file exchange for swarms.

## Storage Migration

This is the most important architectural track.

- [-] Make SurrealDB + RocksDB the default memory backend, not only the swarm-preferred backend.
- [ ] Define a single storage abstraction that covers:
  - long-term memory
  - conversation history
  - sticker/media cache
  - swarm tasks
  - swarm mailbox
  - scheduled jobs
  - gateway/session metadata
- [-] Port current SQLite-only data paths to the unified backend.
- [-] Keep SQLite only as:
  - test/dev fallback, or
  - optional lightweight mode
- [ ] Add migration tooling from SQLite to SurrealDB.
- [ ] Decide how vector search should work in Surreal:
  - native vector index if sufficient
  - external vector sidecar if needed
  - hybrid FTS + vector ranking contract
- [ ] Benchmark Surreal/Rocks against current SQLite flows:
  - chat history load
  - memory insert
  - memory search
  - concurrent swarm coordination

## Gateway Hardening And Scope

- [ ] Keep gateway loopback-only by default.
- [ ] Ensure gateway is only useful through authenticated agent/chat/control-plane flows.
- [ ] Separate low-risk status endpoints from high-risk action endpoints.
- [ ] Add body-size limits, rate limits, and request timeouts.
- [ ] Add audit logging for gateway auth failures and privileged actions.
- [ ] Add explicit support for trusted reverse proxy mode.
- [ ] Add channel-originated capability scoping so chat channels can use the gateway safely without turning it into general remote RCE.

## Agent Loop And Runtime

- [ ] Add explicit round-based compaction thresholds.
- [ ] Compact by both:
  - round count
  - accumulated context size
- [ ] Add cancellation and timeout propagation through subagents and tools.
- [ ] Add loop observer events and progress reporting that are actually consumed.
- [ ] Add better tool-call dedup / loop detection.
- [ ] Add memory-aware context loading so only relevant history is injected.

## Memory And Retrieval

- [ ] Move hybrid retrieval contract to backend-agnostic interfaces.
- [ ] Add score fusion:
  - keyword / FTS
  - vector similarity
  - recency
  - source weighting
- [ ] Add background embedding/index maintenance jobs.
- [ ] Add deduplication / compaction for memory entries.
- [ ] Add admin/debug tooling for memory search quality.

## Performance And Efficiency

- [ ] Profile hot paths after the recent agent-loop and SQLite changes.
- [ ] Remove unnecessary allocations in provider request construction.
- [ ] Reduce message/history copies during compaction and persistence.
- [ ] Rework Telegram voice path to avoid model/process cold start on every request.
- [ ] Add bounded worker pools for DB, transcription, and swarm tasks.
- [ ] Add cheap perf benchmarks for:
  - startup time
  - memory search
  - agent round latency
  - swarm fan-out

## Security And Safety

- [ ] Add centralized dangerous capability taxonomy.
- [ ] Add approval hooks for mutation/execution tools where policy requires it.
- [ ] Add secret scrubbing in logs and event streams.
- [ ] Add better SSRF defenses for metadata/internal-address abuse without blocking normal website access.
- [ ] Add channel allowlist / pairing review and tighten defaults where remote ingress exists.
- [ ] Add plugin/runtime sandbox policy options for deployments that want stricter execution.

## Testing And Validation

- [ ] Add a dedicated regression suite for gateway auth and exposure rules.
- [ ] Add swarm integration tests for task board, mailbox, and delegation limits.
- [ ] Add memory backend conformance tests so SQLite and Surreal behave identically at the trait boundary.
- [ ] Add performance smoke tests in CI.
- [ ] Add release checks for binary size drift.

## Suggested Execution Order

- [x] Phase 1: gateway audit + dangerous-tool registry + doctor command
- [-] Phase 2: SurrealDB as default memory backend + backend migration tooling
- [-] Phase 3: Hermes-style foundations in Rust:
  - toolsets
  - session search
  - managed skills
  - runtime adapter scaffolding
- [ ] Phase 4: compaction v2 + persistent todo/task tools + richer session state
- [ ] Phase 5: swarm task board + mailbox + delegation permissions
- [ ] Phase 6: quality gates + observer/metrics/tracing
- [ ] Phase 7: optional isolated worker runtime + distributed manager/worker mode

## Decisions Needed

- [!] Should SQLite remain supported long-term as a lightweight mode, or be fully demoted to migration/test-only after Surreal parity is complete?
- [!] Should swarm coordination live entirely in SurrealDB, or should some high-contention paths use a separate fast queue/cache?
- [!] Should the first quality-gate implementation be local command-based only, or include agent-evaluator loops from the start?
- [!] Should manager/worker mode be introduced before or after Surreal becomes the primary backend?
