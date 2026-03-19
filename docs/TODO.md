# TODO

Last updated: 2026-03-18

Status key:
- `[ ]` not started
- `[-]` in progress
- `[x]` done

## Verified Done

- [x] SurrealDB memory schema includes `files`, `chunks`, FTS, and sticker cache
- [x] Conversation history is loaded by `chat_id` before agent calls
- [x] `CLAUDE.md` is in sync with repo instructions via symlink to `AGENTS.md`
- [x] `cargo clippy --all-targets -- -D warnings` passes
- [x] `cargo test` passes
- [x] `cargo build --release` passes
- [x] GitHub issue `#2` is resolved and closed
- [x] GitHub issue `#3` is resolved and closed
- [x] GitHub issue `#4` is resolved and closed
- [x] GitHub issue `#5` is resolved and closed
- [x] GitHub issue `#6` is resolved and closed

## Active Work

- [-] tighten docs so they match the actual state of `main`
- [-] keep the device-first branch clearly separated from hosted platform work
- [-] finish tightening SurrealDB docs and scheduler/session coverage
- [-] thread Hermes-style additions into the runtime without bloating the hot
  path:
  - toolset allowlists
  - session search
  - managed skills
  - Daytona runtime adapter

## Next Up

- [ ] add tests that cover Telegram markdown conversion and long-message chunking
- [ ] add tests for audio/sticker handling in Telegram
- [ ] finish porting scheduler/cron/session metadata onto the Surreal contract
- [ ] wire Daytona runtime selection into actual command/tool execution
- [ ] let the agent create and update managed skills automatically after useful
  task completions

## Possible Issues To Watch

- [ ] Discord and WhatsApp still look thin compared with Telegram
- [ ] browser/tool surface is broad; security policy coverage should keep growing
- [ ] swarm and storage layers are moving faster than the user-facing docs
- [ ] keep the default build aligned with the Surreal-backed storage contract
