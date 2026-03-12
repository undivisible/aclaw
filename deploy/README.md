# Deploy

This directory contains starter deployment assets for the hosted gateway mode.

Files:
- `docker-compose.yml`: local hosted gateway + OTEL collector
- `otel-collector.yaml`: local OTLP collector config
- `fly/fly.toml`: Fly.io starter config
- `kubernetes/deployment.yaml`: Kubernetes deployment
- `kubernetes/service.yaml`: Kubernetes service

Notes:
- The gateway expects `UNTHINKCLAW_GATEWAY_TOKEN`.
- The gateway serves Prometheus-style metrics on `/metrics`.
- WebSocket chat is available on `/ws`.
- Session-scoped WebSocket events are available on `/ws/:agent_id`.
