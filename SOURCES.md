# Sources for `rust_tools.json`

This dataset is a curated local knowledge base used by the agent for structured retrieval in Track C.

## What this file contains

The file `rust_tools.json` contains manually structured cards about Rust backend ecosystem tools and libraries.

Each entry includes:
- `id`
- `title`
- `category`
- `tags`
- `summary`
- `use_cases`
- `pros`
- `cons`
- `related`
- `source`
- `collected_at`

## How the data was prepared

The dataset was assembled manually from official project documentation, official project repositories, and project homepages.

The `summary` field is based on official descriptions of the project.

The `pros`, `cons`, `use_cases`, and some `related` fields are curated manually for the purposes of this project. They are not always direct quotes from maintainers and should be treated as practical engineering notes rather than official claims.

## Collection date

Collected and curated on: `2026-04-24`

## Sources by project

- Axum  
  Source type: official docs and official repository  
  Notes: positioning as a web framework / routing library in the Tokio ecosystem

- Actix Web  
  Source type: official docs and official repository  
  Notes: positioning as a high-performance Rust web framework

- Tokio  
  Source type: official docs and official repository  
  Notes: async runtime and ecosystem foundation

- Tower  
  Source type: official docs and ecosystem references  
  Notes: service abstraction and middleware composition

- Hyper  
  Source type: official project site and repository  
  Notes: low-level HTTP library for clients and servers

- Reqwest  
  Source type: crate documentation and repository  
  Notes: ergonomic HTTP client for Rust

- SQLx  
  Source type: official repository and README  
  Notes: async SQL toolkit with compile-time checked queries

- tracing  
  Source type: official repository and docs  
  Notes: structured diagnostics and instrumentation framework

- tonic  
  Source type: official repository and docs  
  Notes: gRPC implementation for Rust

- Serde  
  Source type: official documentation  
  Notes: serialization and deserialization framework

- SeaORM  
  Source type: official documentation  
  Notes: async ORM for Rust

## Purpose in the agent

This dataset is used by the `local_knowledge_search` tool.

Its purpose is to provide:
- curated comparisons
- trade-offs
- common use cases
- lightweight structured retrieval

It complements live data from external APIs and is intentionally small, explainable, and easy to inspect.

## Limitations

- The dataset is small and manually curated.
- It is not a full benchmark database.
- Some trade-offs reflect engineering judgment and may depend on context.
- The dataset may become outdated over time and should be refreshed if the project evolves.
