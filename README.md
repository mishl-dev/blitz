# Blitz

Lightweight HTTP proxy for [chatjimmy.ai](https://chatjimmy.ai) with OpenAI-compatible API.

## Quick Start

```bash
docker-compose up -d
# Server runs on http://localhost:3000
```

## API

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/v1/models` | List models |
| `POST` | `/v1/chat/completions` | Chat (supports `stream: true` for SSE) |

```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"llama3.1-8B","messages":[{"role":"system","content":"You are a pirate"},{"role":"user","content":"Hi"}]}'
```

## Features

- OpenAI-compatible API
- System prompt (first `system` message)
- SSE streaming (`stream: true`)
- Distroless Docker image (~20MB)

## Config

| Env | Default |
|-----|---------|
| `RUST_LOG` | `info` |
| `BIND_ADDR` | `0.0.0.0:3000` |

## License

MIT
