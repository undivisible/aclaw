# Phase 4 Roadmap — Full Feature Parity

**Status**: ✅ Phases 1-3 complete  
**Next**: Phase 4 (optional, 2-3 weeks)

---

## Gaps to Close

### 1. Cost Tracking (from ZeroClaw)
**Impact**: Production monitoring + billing  
**Effort**: ~200 LOC  
**Implementation**:
- Add `cost_per_token` to Provider trait
- Track tokens in AgentRunner
- Cost table in SQLite (namespace, model, tokens, cost, timestamp)
- GET `/api/cost/summary` endpoint
- GET `/api/cost/history` with date range

**Example**:
```bash
curl http://localhost:8080/api/cost/summary
{
  "total_cost": 2.34,
  "by_model": {
    "claude-opus-4-6": 1.89,
    "gpt-4": 0.45
  },
  "current_month": 18.50
}
```

### 2. Cron Scheduler (from ZeroClaw)
**Impact**: Autonomous recurring tasks  
**Effort**: ~300 LOC  
**Implementation**:
- `Schedule` struct (cron expression, task_id, enabled)
- Tokio interval + cron parser
- POST `/api/schedule` (create recurring task)
- GET `/api/schedule` (list schedules)
- DELETE `/api/schedule/{schedule_id}`
- Storage in SQLite (schedules table)

**Example**:
```bash
curl -X POST http://localhost:8080/api/schedule \
  -d '{
    "cron": "0 9 * * MON",
    "task_goal": "Review Monday digest",
    "priority": 7
  }'
```

### 3. Additional Channels
**Impact**: User reach + multi-platform  
**Effort**: ~150 LOC per channel  
**Channels to add**:
- Matrix (already skeleton)
- Slack (webhook + bot API)
- WhatsApp (via Twilio)
- IRC (for nostalgia)

### 4. Container Security Hardening (from NanoClaw)
**Impact**: Multi-tenant safety  
**Effort**: ~400 LOC  
**Implementation**:
- IPC auth (like NanoClaw)
- Sender allowlist per container
- Mount security (restrict volumes)
- Network isolation (no external access by default)
- SELinux/AppArmor support

### 5. Tool Expansion
**Impact**: Agent capability + utility  
**Effort**: ~100 LOC per tool  
**Tools to add**:
- Image analysis (vision, like ZeroClaw)
- HTTP requests (custom headers, auth)
- Screenshot (hardware, like ZeroClaw)
- Email (SMTP, IMAP via himalaya)
- Database query (SQL adapter)

### 6. LLM Streaming Responses
**Impact**: Real-time token output  
**Effort**: ~200 LOC  
**Implementation**:
- Provider trait: `stream()` method (returns StreamReceiver)
- Anthropic streaming (native)
- OpenAI streaming (native)
- Ollama streaming (native)
- WebSocket chunk forwarding
- SSE response headers for HTTP

---

## Phase 4 Implementation Order

1. **Week 1: Cost + Cron** (critical for production monitoring)
2. **Week 2: Security hardening** (if multi-tenant needed)
3. **Week 3: Tools + channels** (feature completeness)
4. **Ongoing: LLM streaming** (nice-to-have, high ROI)

---

## Phase 4 Binary Impact

Current: 4.2MB  
With cost + cron: ~4.3MB  
With all Phase 4: ~4.5MB (still tiny)

---

## Production Deployment Today (Phase 3)

✅ **Ready to use**:
- Single-agent chat (CLI, Telegram, Discord)
- HTTP/WebSocket gateway
- Agent swarms (Vibemania)
- Plugin system
- Vector embeddings
- Claw migration

⏳ **Can add later** (Phase 4):
- Cost tracking (for billing)
- Cron scheduler (for automation)
- Extra channels (for reach)
- Security hardening (if multi-tenant)

---

**Recommendation**: Deploy Phase 3 now. Add Phase 4 features as business needs arise.

---

Generated: 2026-03-07  
Status: Ready
