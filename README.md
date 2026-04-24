Track: A+C

# Rust AI Agent

An agentic assistant for exploring the Rust backend ecosystem.

The project combines:
- **Track A** — an external API tool via **GitHub API**
- **Track C** — a structured local data tool via a curated **JSON knowledge base**

The agent accepts a user query, decides whether tools are needed, calls them when appropriate, and synthesizes a final answer.

## What the agent does

The agent is designed for questions like:

- “Compare Axum and Actix for a new Rust backend”
- “Compare Axum and Actix and show active GitHub repos”
- “Find active Rust observability repositories”
- “What tools are good for a Tokio-based backend?”

It supports:
- tool planning
- tool execution
- session memory
- final answer synthesis

## Why this project

I wanted to build a small but real agent system rather than just a chatbot wrapper around an LLM.

This project focuses on:
- explicit orchestration
- real tool usage
- explainable architecture
- practical trade-offs between local structured knowledge and live external data

The chosen domain is the Rust backend ecosystem, because it is both personally interesting and technically rich enough for meaningful retrieval and comparison tasks.

---

## Architecture

```text
Streamlit UI
   ↓
Rust Backend (Axum)
   ↓
Agent Orchestrator
   ├─ Planner
   ├─ Tool 1: local_knowledge_search
   ├─ Tool 2: github_search
   └─ Synthesizer
   ↓
Ollama
   ↓
Redis session storage
