# Roadmap

Last updated: 2026-03-18

## Now

- keep the local-first runtime stable while `main` stays focused on the
  single-machine experience
- finish the storage migration posture so SurrealDB + RocksDB is the primary
  path and SQLite is just the fallback
- start wiring Hermes-inspired runtime pieces into the actual execution path:
  toolsets, managed skills, session search, and runtime adapters
- add more regression tests around channels and provider retry behavior

## Next

- improve gateway hardening and operator diagnostics
- make tool-policy boundaries easier to audit
- add stronger tests around channels, gateway auth, and loop behavior
- make swarm state more visible and easier to operate
- wire Daytona-style isolated runtime execution behind a Rust runtime adapter
- move scheduler/session/control metadata onto the unified storage contract

## Later

- unify storage contracts for memory, session state, swarm tasks, and scheduling
- improve compaction, context loading, and persistent task planning
- add better observability without bloating the binary
- add agent-authored skill maintenance and better long-term procedural memory

## Not For This Branch

- hosted gateway product work
- web UI and deployment surface
- multi-user control-plane features that belong on `codex/full-platform`
