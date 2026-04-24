# Demo Script

## Query 1

Prompt:
`I need Rust libraries for metrics, tracing, and log correlation in a production API. Pick a practical stack for me.`

Expected behavior:
- Should use `crates_search`
- May also use `local_knowledge_search` if the planner wants extra guidance
- Answer should recommend a coherent observability stack, not just a random list

## Query 2

Prompt:
`Give me Rust Axum GitHub repo and tell about it`

Expected behavior:
- Should use `github_search`, likely together with `local_knowledge_search`
- Answer should mention repo information plus a short trade-off summary and some data extracted from context of the session
