# Rust AI Agent

An agentic assistant for exploring the Rust backend ecosystem.

This project combines three retrieval sources:
- `local_knowledge_search`: a curated JSON knowledge base for stable framework trade-offs
- `github_search`: live GitHub repository discovery
- `crates_search`: crates.io ecosystem search for concrete Rust libraries

The agent receives a user question, decides which tools to call, executes them, and synthesizes a final answer with session memory.

## What It Can Answer

Typical prompts:
- `Compare Axum and Actix for a new Rust backend`
- `Compare Axum and Actix and show active GitHub repos`
- `Find active Rust observability repositories`
- `I'm building a Rust service with layered config from files, env vars, and secrets. Which crates should I evaluate?`
- `I need Rust libraries for metrics, tracing, and log correlation in a production API`

## Stack

- Backend: Rust, Axum, Tokio
- Frontend: Streamlit
- LLM runtime: Ollama
- Session storage: Redis
- Live APIs:
  - GitHub Search API
  - crates.io API

## Architecture

```text
Streamlit UI
   ↓
Rust Backend (Axum)
   ↓
Agent Orchestrator
   ├─ Fast-path policy
   ├─ Planner
   ├─ local_knowledge_search
   ├─ github_search
   ├─ crates_search
   └─ Synthesizer
   ↓
Ollama
   ↓
Redis session storage
```

## Repository Layout

```text
backend/
  data/rust_tools.json         Curated local knowledge base
  src/agent/                   Planner, policy, synthesizer, orchestrator
  src/tools/                   GitHub, local, and crates tools
  src/api/                     HTTP handlers and debug routes

frontend/
  app.py                       Streamlit UI

demo/
  demo_script.md               Suggested demo prompts

docker-compose.yaml            Default full-stack Compose
docker-compose.macos-host-ollama.yaml
docker-compose.nvidia-gpu.yaml
.env.example                   Environment template
```

## Supported Run Modes

This project is primarily documented for Docker-based runs.

### 1. Default Docker Compose

Use this when you want the whole stack in containers:
- Redis
- Ollama
- backend
- frontend

```bash
cp .env.example .env
docker compose up --build
```

Open:
- Frontend: [http://localhost:8501](http://localhost:8501)
- Backend health: [http://localhost:8080/health](http://localhost:8080/health)

### 2. macOS with Ollama Running Natively on Host

Use this on Apple Silicon if you want Ollama to use Metal on the host instead of running inside Docker.

1. Copy env file:

```bash
cp .env.example .env
```

2. Start Ollama on the host:

```bash
ollama serve
```

3. Start the rest of the stack:

```bash
docker compose -f docker-compose.macos-host-ollama.yaml up --build
```

This mode runs:
- Redis in Docker
- backend in Docker
- frontend in Docker
- Ollama on the macOS host via `http://host.docker.internal:11434`

### 3. Docker Compose with NVIDIA GPU for Ollama

Use this on a Linux machine with an NVIDIA GPU such as an RTX 4060.

Requirements:
- Linux host
- NVIDIA driver installed
- [NVIDIA Container Toolkit](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html)

Run:

```bash
cp .env.example .env
docker compose -f docker-compose.yaml -f docker-compose.nvidia-gpu.yaml up --build
```

This keeps the default architecture but gives the `ollama` container access to NVIDIA GPU resources.

## Environment Setup

Create a local env file:

```bash
cp .env.example .env
```

Current template:

```env
GITHUB_TOKEN=YOUR_PERSONAL_ACCESS_TOKEN_HERE
HOST=0.0.0.0
PORT=8080
REDIS_URL=redis://redis:6379
SESSION_TTL_SECS=86400
OLLAMA_BASE_URL=http://ollama:11434
OLLAMA_PLANNER_MODEL=qwen3:8b
OLLAMA_SYNTHESIZER_MODEL=qwen3:8b
OLLAMA_PLANNER_THINKING=false
OLLAMA_SYNTHESIZER_THINKING=true
OLLAMA_KEEP_ALIVE=10m
CRATES_API_BASE_URL=https://crates.io/api/v1
CRATES_API_USER_AGENT=ai-rust-agent (replace-with-contact@example.com)
CRATES_API_RATE_LIMIT_MS=1000
RUST_LOG=debug,tower_http=debug
```

## `.env` Fields Explained

### `GITHUB_TOKEN`

GitHub personal access token used by `github_search`.

- Required: no, but strongly recommended
- Why: unauthenticated GitHub search is much easier to rate-limit
- What to put:

```env
GITHUB_TOKEN=ghp_your_token_here
```

If you do not want to use GitHub search, you can leave it empty:

```env
GITHUB_TOKEN=
```

### `HOST`

Backend bind host.

- Default: `0.0.0.0`
- Usually keep as-is

### `PORT`

Backend HTTP port.

- Default: `8080`
- Frontend expects backend on this port unless you change frontend config too

### `REDIS_URL`

Redis connection string used for session storage.

- Default in Docker: `redis://redis:6379`
- Typical local Redis value: `redis://127.0.0.1:6379`

### `SESSION_TTL_SECS`

How long chat sessions stay in Redis.

- Default: `86400` seconds = 24 hours

### `OLLAMA_BASE_URL`

Base URL for the Ollama HTTP API.

Use one of these values:
- Default Docker Compose: `http://ollama:11434`
- Host Ollama on macOS: `http://host.docker.internal:11434`
- Fully local host run: `http://localhost:11434`

The backend accepts this under the `OLLAMA_BASE_URL` name. Internally it is loaded into `ollama_url`.

### `OLLAMA_PLANNER_MODEL`

Model used by the planner.

Recommended values:
- `qwen3:8b` for safer planner quality
- `qwen3:4b` if you want a faster but weaker planner

### `OLLAMA_SYNTHESIZER_MODEL`

Model used by the synthesizer.

Recommended value:
- `qwen3:8b`

### `OLLAMA_PLANNER_THINKING`

Enables or disables reasoning mode for the planner model.

- Recommended: `false`
- Why: planner mainly does routing and query shaping; disabling thinking usually lowers latency

### `OLLAMA_SYNTHESIZER_THINKING`

Enables or disables reasoning mode for the synthesizer model.

- Recommended default: `true`
- If latency matters more than answer quality, you can try `false`

### `OLLAMA_KEEP_ALIVE`

How long Ollama keeps the model warm.

- Default: `10m`
- Good values: `5m`, `10m`, `30m`

### `CRATES_API_BASE_URL`

Base URL for crates.io API.

- Default: `https://crates.io/api/v1`
- Usually do not change this

### `CRATES_API_USER_AGENT`

Descriptive user agent for crates.io requests.

- Required in practice: yes
- API key needed: no
- What to put: a real app name plus contact email or URL

Recommended format:

```env
CRATES_API_USER_AGENT=matmod-rust-agent (your-email@example.com)
```

or

```env
CRATES_API_USER_AGENT=matmod-rust-agent (https://github.com/your-name)
```

### `CRATES_API_RATE_LIMIT_MS`

Delay between crates.io requests.

- Default: `1000`
- Leave it as-is unless you know what you are doing

### `RUST_LOG`

Rust tracing filter for backend logs.

Useful values:

```env
RUST_LOG=info,tower_http=debug
```

```env
RUST_LOG=debug,tower_http=debug
```

## Recommended `.env` Examples

### Default Docker

```env
GITHUB_TOKEN=ghp_your_token_here
HOST=0.0.0.0
PORT=8080
REDIS_URL=redis://redis:6379
SESSION_TTL_SECS=86400
OLLAMA_BASE_URL=http://ollama:11434
OLLAMA_PLANNER_MODEL=qwen3:8b
OLLAMA_SYNTHESIZER_MODEL=qwen3:8b
OLLAMA_PLANNER_THINKING=false
OLLAMA_SYNTHESIZER_THINKING=true
OLLAMA_KEEP_ALIVE=10m
CRATES_API_BASE_URL=https://crates.io/api/v1
CRATES_API_USER_AGENT=matmod-rust-agent (your-email@example.com)
CRATES_API_RATE_LIMIT_MS=1000
RUST_LOG=debug,tower_http=debug
```

### macOS Host Ollama

```env
GITHUB_TOKEN=ghp_your_token_here
HOST=0.0.0.0
PORT=8080
REDIS_URL=redis://redis:6379
SESSION_TTL_SECS=86400
OLLAMA_BASE_URL=http://host.docker.internal:11434
OLLAMA_PLANNER_MODEL=qwen3:8b
OLLAMA_SYNTHESIZER_MODEL=qwen3:8b
OLLAMA_PLANNER_THINKING=false
OLLAMA_SYNTHESIZER_THINKING=true
OLLAMA_KEEP_ALIVE=10m
CRATES_API_BASE_URL=https://crates.io/api/v1
CRATES_API_USER_AGENT=matmod-rust-agent (your-email@example.com)
CRATES_API_RATE_LIMIT_MS=1000
RUST_LOG=debug,tower_http=debug
```

## API Overview

### Main Routes

- `GET /health`
- `POST /sessions`
- `GET /history/{session_id}`
- `POST /reset/{session_id}`
- `POST /chat`

### Debug Routes

- `POST /debug/llm`
- `POST /debug/plan`
- `POST /debug/local-search`
- `POST /debug/github-search`
- `POST /debug/crates-search`
- `POST /debug/execute`

## Minimal API Examples

Create session:

```bash
curl -X POST http://localhost:8080/sessions
```

Chat:

```bash
curl -X POST http://localhost:8080/chat \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "PUT-SESSION-ID-HERE",
    "message": "Compare Axum and Actix and show active GitHub repos"
  }'
```

Health:

```bash
curl http://localhost:8080/health
```

## Frontend

The Streamlit frontend lives in [frontend/app.py](frontend/app.py).

Features:
- session creation
- reset and reload history
- chat interface
- last-used tool display
- simple error reporting

Default frontend URL:
- [http://localhost:8501](http://localhost:8501)

## Agent Behavior

High-level flow:
1. Save user message in Redis session
2. Apply smalltalk fast-path if appropriate
3. Run planner
4. Filter and normalize tool calls in policy
5. Execute tools
6. Synthesize final answer
7. Save assistant response back to session history

Important runtime behavior:
- planner and synthesizer can use different Ollama models
- independent tools are executed in parallel when possible
- partial answers are supported when one tool fails
- recent chat history is passed into the synthesizer
- GitHub queries are normalized to short keyword searches
- crates.io results are expanded and reranked to improve retrieval quality

## Notes About Local Development

The primary supported workflow is Docker Compose.

The backend currently loads the local knowledge base from `/app/data/rust_tools.json`, which matches the container layout. Because of that, a pure host-native backend run is not the main documented path right now unless you also mirror that path locally or adjust the code.

If you still want to run only the frontend locally:

```bash
cd frontend
uv sync
uv run streamlit run app.py
```

and point it at a running backend with:

```bash
BACKEND_URL=http://localhost:8080
```

## Demo

Suggested prompts are in [demo/demo_script.md](demo/demo_script.md).

## Troubleshooting

### `chat_handler failed: ... http://ollama:11434 ... dns error`

Your backend cannot reach Ollama.

Check:
- in default Docker mode, `ollama` container should be running
- in macOS host mode, use `docker-compose.macos-host-ollama.yaml`
- make sure `OLLAMA_BASE_URL` points to the correct host

### GitHub search returns rate-limit errors or empty results

Set `GITHUB_TOKEN` in `.env`.

### crates.io results are weak or empty

Check:
- `CRATES_API_USER_AGENT` is set to a descriptive real value
- outbound network access is available
- `CRATES_API_BASE_URL` is still `https://crates.io/api/v1`

### Frontend loads but answers fail

Check:
- backend is healthy at `/health`
- frontend can reach `BACKEND_URL`
- Redis is running
- Ollama has the configured models pulled

## Validation

Useful local checks:

```bash
cd backend
cargo fmt
cargo check
```

## License

No license file is included in this repository at the moment.
