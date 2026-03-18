# Swarm Upgrade Notes

Last updated: 2026-03-18

This file is the clean migration note for finishing the swarm/storage upgrade.
It replaces the older design-dump version.

## Goal

Finish the move from mixed local runtime storage to a SurrealDB + RocksDB-first
system that can support:

- shared swarm state
- session and conversation retrieval
- cache-heavy local data
- agent coordination without bolting on separate ad hoc stores

## Migration Order

1. Make the storage contract consistent.
2. Port SQLite-only scheduler/session paths.
3. Keep SQLite only as fallback or remove it entirely.
4. Push runtime and swarm to the same backend expectations.
5. Add migration tooling and backend conformance tests.

## Must-Have Work

- unify memory, session, scheduler, and cache expectations at the trait boundary
- move cron/scheduler persistence off direct SQLite assumptions
- move runtime conversation/state loading onto the same backend posture as swarm
- add migration tooling for existing local data
- add parity tests so storage behavior is consistent across backends

## Nice-To-Have Work

- better local cache policy for embeddings/chunks/media
- more visible swarm status and operator tooling
- stronger observability around delegation and task state

## Non-Goals For This File

- detailed future product design
- speculative distributed topologies
- placeholder schema sketches that do not match the codebase yet

Use [CLAW-ADOPTION-TODO.md](CLAW-ADOPTION-TODO.md)
for the long backlog and [SWARM.md](SWARM.md)
for the current feature snapshot.
