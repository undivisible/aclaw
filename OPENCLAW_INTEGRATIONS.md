# OpenClaw Integrations to Port

## Channels (21 total)
OpenClaw has 21 messaging channel integrations:

| # | Channel | Priority | Complexity | Notes |
|---|---------|----------|-----------|-------|
| 1 | **telegram** | ✅ DONE | Medium | Polling-based, already in aclaw |
| 2 | **discord** | ✅ DONE | Medium | HTTP API, already in aclaw |
| 3 | **slack** | HIGH | Medium | Bot + App tokens, Web API |
| 4 | **whatsapp** | HIGH | High | Baileys library, QR auth |
| 5 | **signal** | HIGH | High | signal-cli bridge |
| 6 | **matrix** | MEDIUM | Medium | Matrix SDK, E2E encryption |
| 7 | **irc** | MEDIUM | Low | Simple protocol |
| 8 | **imessage** | MEDIUM | Medium | macOS only (BlueBubbles) |
| 9 | **bluebubbles** | MEDIUM | Medium | iMessage bridge |
| 10 | **googlechat** | MEDIUM | Medium | Google Workspace API |
| 11 | **msteams** | MEDIUM | High | Microsoft Bot Framework |
| 12 | **mattermost** | LOW | Medium | Slack-like API |
| 13 | **nextcloud-talk** | LOW | Medium | Nextcloud API |
| 14 | **feishu** | LOW | Medium | Lark/Feishu API |
| 15 | **line** | LOW | Medium | LINE Messaging API |
| 16 | **nostr** | LOW | Medium | Decentralized protocol |
| 17 | **synology-chat** | LOW | Low | Synology webhook |
| 18 | **tlon** | LOW | High | Urbit/Tlon |
| 19 | **twitch** | LOW | Medium | IRC-based + API |
| 20 | **zalo** | LOW | Medium | Vietnam messaging |
| 21 | **zalouser** | LOW | Medium | Zalo user API |

## LLM Providers (20+ total)
| # | Provider | Priority | Base URL | Auth |
|---|----------|----------|----------|------|
| 1 | **anthropic** | ✅ DONE | api.anthropic.com | API key + OAuth |
| 2 | **openai** | ✅ DONE | api.openai.com | API key |
| 3 | **ollama** | ✅ DONE | localhost:11434 | None |
| 4 | **openrouter** | ✅ DONE | openrouter.ai | API key |
| 5 | **groq** | ✅ DONE | api.groq.com | API key |
| 6 | **google/gemini** | ✅ DONE | generativelanguage.googleapis.com | API key |
| 7 | **github-copilot** | HIGH | api.individual.githubcopilot.com | Token exchange |
| 8 | **amazon-bedrock** | HIGH | AWS SDK | AWS credentials |
| 9 | **google-vertex** | MEDIUM | us-central1-aiplatform.googleapis.com | Service account |
| 10 | **mistral** | MEDIUM | api.mistral.ai | API key |
| 11 | **deepseek** | MEDIUM | api.deepseek.com | API key |
| 12 | **together** | MEDIUM | api.together.xyz | API key |
| 13 | **fireworks** | MEDIUM | api.fireworks.ai | API key |
| 14 | **perplexity** | MEDIUM | api.perplexity.ai | API key |
| 15 | **xai** | MEDIUM | api.x.ai | API key |
| 16 | **moonshot/kimi** | LOW | api.moonshot.ai | API key |
| 17 | **minimax** | LOW | api.minimax.io | API key + OAuth |
| 18 | **venice** | LOW | api.venice.ai | API key |
| 19 | **synthetic** | LOW | api.synthetic.ai | API key |
| 20 | **kilocode** | LOW | kilocode API | API key |
| 21 | **huggingface** | LOW | huggingface.co | API key |
| 22 | **cloudflare-ai** | LOW | Cloudflare Workers AI | API key |
| 23 | **siliconflow** | LOW | siliconflow.cn | API key |
| 24 | **volcengine** | LOW | volcengine.com | API key |
| 25 | **xiaomi** | LOW | xiaomi API | API key |
| 26 | **qwen** | LOW | qwen API | OAuth |
| 27 | **zai** | LOW | z.ai | API key |
| 28 | **vercel-ai-gateway** | LOW | Vercel AI Gateway | API key |
| 29 | **cerebras** | LOW | cerebras API | API key |

## Auth Providers (4)
| # | Auth | Notes |
|---|------|-------|
| 1 | google-antigravity-auth | Google OAuth for Gemini CLI |
| 2 | google-gemini-cli-auth | Gemini CLI token refresh |
| 3 | minimax-portal-auth | MiniMax OAuth |
| 4 | qwen-portal-auth | Qwen OAuth |

## Tools/Features (14)
| # | Feature | Priority | Notes |
|---|---------|----------|-------|
| 1 | **memory-core** | HIGH | Core memory system |
| 2 | **memory-lancedb** | MEDIUM | Vector DB (LanceDB) |
| 3 | **voice-call** | MEDIUM | Voice calls (ElevenLabs) |
| 4 | **talk-voice** | MEDIUM | Voice processing |
| 5 | **phone-control** | MEDIUM | Phone automation |
| 6 | **device-pair** | MEDIUM | Device pairing |
| 7 | **copilot-proxy** | HIGH | GitHub Copilot relay |
| 8 | **acpx** | HIGH | ACP agent execution |
| 9 | **diffs** | MEDIUM | Code diff handling |
| 10 | **lobster** | LOW | Workflow orchestration |
| 11 | **llm-task** | MEDIUM | Generic LLM task runner |
| 12 | **open-prose** | LOW | Prose editing |
| 13 | **thread-ownership** | MEDIUM | Thread management |
| 14 | **diagnostics-otel** | LOW | OpenTelemetry diagnostics |

## Implementation Plan

### Phase 1: Critical Providers (This Session)
- [x] Anthropic (OAuth + API key)
- [ ] GitHub Copilot (token exchange via proxy)
- [ ] Gemini CLI auth (google-antigravity)
- [ ] OpenAI Codex (responses API)

### Phase 2: High-Priority Channels
- [ ] Slack (Web API + Events API)
- [ ] WhatsApp (Baileys)
- [ ] Signal (signal-cli)
- [ ] Matrix (matrix-sdk)

### Phase 3: More Providers
- [ ] Amazon Bedrock (AWS SDK)
- [ ] Mistral
- [ ] DeepSeek
- [ ] Together
- [ ] Fireworks
- [ ] Perplexity
- [ ] xAI/Grok

### Phase 4: Remaining Channels
- [ ] IRC
- [ ] Google Chat
- [ ] MS Teams
- [ ] Mattermost
- [ ] Line
- [ ] Nostr
- [ ] Twitch

### Phase 5: Features
- [ ] Voice calls (ElevenLabs)
- [ ] ACP execution
- [ ] LanceDB vectors
- [ ] Workflow orchestration
