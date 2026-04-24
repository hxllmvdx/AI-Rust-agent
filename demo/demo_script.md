# Demo Script

## Query 1

Prompt:
`I'm building a Rust service with layered config from files, env vars, and secrets. Which crates should I evaluate, and what are the trade-offs?`

Expected behavior:
- Should use `crates_search`, likely together with `local_knowledge_search`
- Answer should mention concrete crates plus a short trade-off summary

## Query 2

Prompt:
`I need Rust libraries for metrics, tracing, and log correlation in a production API. Pick a practical stack for me.`

Expected behavior:
- Should use `crates_search`
- May also use `local_knowledge_search` if the planner wants extra guidance
- Answer should recommend a coherent observability stack, not just a random list
