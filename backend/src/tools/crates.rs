use std::{collections::HashSet, sync::Arc, time::Duration};

use reqwest::Client;
use tokio::{
    sync::Mutex,
    time::{Instant, sleep},
};

use crate::{
    error::BackendError,
    models::crates::{
        CrateDetailResponse, CrateSearchItem, CratesSearchResponse, CratesSearchResult,
    },
};

#[derive(Clone)]
pub struct CratesTool {
    http: Client,
    base_url: String,
    user_agent: String,
    rate_limit: Duration,
    last_request_at: Arc<Mutex<Option<Instant>>>,
}

impl CratesTool {
    pub fn new(base_url: String, user_agent: String, rate_limit: Duration) -> Self {
        Self {
            http: Client::new(),
            base_url,
            user_agent,
            rate_limit,
            last_request_at: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<CratesSearchResult>, BackendError> {
        let mut items = self.collect_candidates(query, limit).await?;
        rerank_crates(query, &mut items);

        let mut results = Vec::with_capacity(limit);
        for item in items.into_iter().take(limit) {
            let enriched = self.enrich_result(item).await?;
            results.push(enriched);
        }

        Ok(results)
    }

    async fn collect_candidates(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<CrateSearchItem>, BackendError> {
        let candidate_queries = build_candidate_queries(query);
        let per_query_limit = (limit * 2).clamp(5, 10);
        let mut seen = HashSet::new();
        let mut merged = Vec::new();

        for candidate in candidate_queries {
            let response = self
                .get::<CratesSearchResponse>(
                    "/crates",
                    &[
                        ("q", candidate.as_str()),
                        ("per_page", &per_query_limit.to_string()),
                    ],
                )
                .await?;

            for item in response.crates {
                if seen.insert(item.id.clone()) {
                    merged.push(item);
                }
            }

            if merged.len() >= limit * 4 {
                break;
            }
        }

        Ok(merged)
    }

    async fn enrich_result(
        &self,
        item: CrateSearchItem,
    ) -> Result<CratesSearchResult, BackendError> {
        let (categories, keywords) = if item.categories.as_ref().is_some_and(|x| !x.is_empty())
            || item.keywords.as_ref().is_some_and(|x| !x.is_empty())
        {
            (
                item.categories.unwrap_or_default(),
                item.keywords.unwrap_or_default(),
            )
        } else {
            let details = self
                .get::<CrateDetailResponse>(&format!("/crates/{}", item.id), &[])
                .await?;
            (
                details.categories.into_iter().map(|x| x.id).collect(),
                details
                    .keywords
                    .into_iter()
                    .map(|x| {
                        if x.keyword.is_empty() {
                            x.id
                        } else {
                            x.keyword
                        }
                    })
                    .collect(),
            )
        };

        Ok(CratesSearchResult {
            name: item.name,
            description: item.description,
            downloads: item.downloads,
            latest_version: item.max_version,
            categories,
            keywords,
        })
    }

    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, BackendError> {
        self.wait_for_rate_limit().await;

        let url = format!("{}{}", self.base_url.trim_end_matches('/'), path);
        let response = self
            .http
            .get(url)
            .header("User-Agent", &self.user_agent)
            .query(query)
            .send()
            .await?
            .error_for_status()?
            .json::<T>()
            .await?;

        Ok(response)
    }

    async fn wait_for_rate_limit(&self) {
        let mut last_request_at = self.last_request_at.lock().await;
        if let Some(previous) = *last_request_at {
            let elapsed = previous.elapsed();
            if elapsed < self.rate_limit {
                sleep(self.rate_limit - elapsed).await;
            }
        }
        *last_request_at = Some(Instant::now());
    }
}

fn rerank_crates(query: &str, items: &mut [CrateSearchItem]) {
    let query_tokens = normalized_tokens(query);
    let role_hints = detect_role_hints(&query_tokens);
    items.sort_by(|left, right| {
        let left_score = crate_score(left, &query_tokens, &role_hints);
        let right_score = crate_score(right, &query_tokens, &role_hints);
        right_score
            .cmp(&left_score)
            .then_with(|| right.downloads.cmp(&left.downloads))
            .then_with(|| left.name.cmp(&right.name))
    });
}

fn crate_score(item: &CrateSearchItem, query_tokens: &[String], role_hints: &[RoleHint]) -> i64 {
    let name_tokens = normalized_tokens(&item.name);
    let description_tokens = normalized_tokens(item.description.as_deref().unwrap_or_default());
    let category_tokens = item
        .categories
        .clone()
        .unwrap_or_default()
        .into_iter()
        .flat_map(|value| normalized_tokens(&value))
        .collect::<Vec<_>>();
    let keyword_tokens = item
        .keywords
        .clone()
        .unwrap_or_default()
        .into_iter()
        .flat_map(|value| normalized_tokens(&value))
        .collect::<Vec<_>>();

    let mut score = 0i64;
    for token in query_tokens {
        if name_tokens.iter().any(|value| value == token) {
            score += 8;
        }
        if description_tokens.iter().any(|value| value == token) {
            score += 4;
        }
        if keyword_tokens.iter().any(|value| value == token) {
            score += 6;
        }
        if category_tokens.iter().any(|value| value == token) {
            score += 5;
        }
    }

    let searchable = combined_tokens(
        &name_tokens,
        &description_tokens,
        &category_tokens,
        &keyword_tokens,
    );
    for role in role_hints {
        let overlap = role
            .tokens
            .iter()
            .filter(|token| searchable.contains(*token))
            .count() as i64;
        score += overlap * role.weight;

        if role
            .preferred_names
            .iter()
            .any(|name| item.name.eq_ignore_ascii_case(name))
        {
            score += role.weight * 6;
        } else if role.preferred_names.iter().any(|name| {
            let preferred_tokens = normalized_tokens(name);
            preferred_tokens
                .iter()
                .all(|token| name_tokens.iter().any(|value| value == token))
        }) {
            score += role.weight * 3;
        }
    }

    if name_tokens
        .iter()
        .any(|token| query_tokens.iter().any(|query| query == token))
    {
        score += 6;
    }

    score + ((item.downloads as f64).ln_1p() * 2.0) as i64
}

fn normalized_tokens(input: &str) -> Vec<String> {
    input
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_string())
        .collect()
}

fn build_candidate_queries(input: &str) -> Vec<String> {
    let tokens = normalized_tokens(input);
    let hints = detect_role_hints(&tokens);
    let mut queries = Vec::new();
    let mut seen = HashSet::new();

    let primary = tokens.iter().take(5).cloned().collect::<Vec<_>>().join(" ");
    push_query(&mut queries, &mut seen, primary);

    let without_rust = tokens
        .iter()
        .filter(|token| token.as_str() != "rust")
        .take(4)
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    push_query(&mut queries, &mut seen, without_rust);

    for hint in &hints {
        let focused = hint
            .tokens
            .iter()
            .take(4)
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");
        push_query(&mut queries, &mut seen, focused);

        let with_rust = std::iter::once("rust".to_string())
            .chain(hint.tokens.iter().take(3).cloned())
            .collect::<Vec<_>>()
            .join(" ");
        push_query(&mut queries, &mut seen, with_rust);

        for candidate in &hint.queries {
            push_query(&mut queries, &mut seen, candidate.clone());

            let with_rust = format!("rust {candidate}");
            push_query(&mut queries, &mut seen, with_rust);
        }
    }

    if hints.is_empty() {
        push_query(
            &mut queries,
            &mut seen,
            std::iter::once("rust".to_string())
                .chain(tokens.iter().take(3).cloned())
                .collect::<Vec<_>>()
                .join(" "),
        );
    }

    if queries.is_empty() {
        push_query(&mut queries, &mut seen, "rust library".to_string());
    }

    queries
}

fn push_query(queries: &mut Vec<String>, seen: &mut HashSet<String>, query: String) {
    let normalized = query.trim().to_string();
    if normalized.is_empty() {
        return;
    }

    if seen.insert(normalized.clone()) {
        queries.push(normalized);
    }
}

fn combined_tokens(
    name_tokens: &[String],
    description_tokens: &[String],
    category_tokens: &[String],
    keyword_tokens: &[String],
) -> std::collections::HashSet<String> {
    name_tokens
        .iter()
        .chain(description_tokens.iter())
        .chain(category_tokens.iter())
        .chain(keyword_tokens.iter())
        .cloned()
        .collect()
}

#[derive(Clone)]
struct RoleHint {
    weight: i64,
    tokens: Vec<String>,
    queries: Vec<String>,
    preferred_names: Vec<String>,
}

fn detect_role_hints(query_tokens: &[String]) -> Vec<RoleHint> {
    let mut hints = Vec::new();

    if has_any_token(
        query_tokens,
        &["config", "configuration", "env", "environment", "secret"],
    ) {
        hints.push(RoleHint {
            weight: 7,
            tokens: vec![
                "config",
                "configuration",
                "figment",
                "confique",
                "dotenvy",
                "secrecy",
                "settings",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            queries: vec![
                "config",
                "figment",
                "confique",
                "dotenvy",
                "secrecy",
                "config secret",
                "configuration",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            preferred_names: vec!["config", "figment", "confique", "dotenvy", "secrecy"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        });
    }

    if has_any_token(
        query_tokens,
        &[
            "metrics",
            "metric",
            "tracing",
            "trace",
            "logging",
            "correlation",
            "observability",
            "telemetry",
        ],
    ) {
        hints.push(RoleHint {
            weight: 8,
            tokens: vec![
                "metrics",
                "metric",
                "tracing",
                "subscriber",
                "opentelemetry",
                "logging",
                "log",
                "telemetry",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            queries: vec![
                "tracing",
                "tracing subscriber",
                "tracing opentelemetry",
                "metrics",
                "metrics exporter prometheus",
                "opentelemetry",
                "opentelemetry otlp",
                "log",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            preferred_names: vec![
                "tracing",
                "tracing-subscriber",
                "metrics",
                "metrics-exporter-prometheus",
                "opentelemetry",
                "opentelemetry-otlp",
                "tracing-opentelemetry",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        });
    }

    if has_any_token(
        query_tokens,
        &[
            "database",
            "databases",
            "sql",
            "orm",
            "postgres",
            "mysql",
            "sqlite",
        ],
    ) {
        hints.push(RoleHint {
            weight: 8,
            tokens: vec![
                "sql", "orm", "sqlx", "diesel", "seaorm", "postgres", "mysql", "sqlite", "database",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            queries: vec!["sqlx", "diesel", "sea orm", "orm", "database"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            preferred_names: vec!["sqlx", "diesel", "sea-orm", "sea-query"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        });
    }

    if has_any_token(
        query_tokens,
        &["cli", "command", "parsing", "terminal", "console"],
    ) {
        hints.push(RoleHint {
            weight: 7,
            tokens: vec!["clap", "bpaf", "argh", "terminal", "tui", "ratatui"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            queries: vec!["clap", "bpaf", "argh", "ratatui", "terminal"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            preferred_names: vec!["clap", "bpaf", "argh", "ratatui"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        });
    }

    if has_any_token(
        query_tokens,
        &[
            "auth",
            "authentication",
            "oauth",
            "jwt",
            "session",
            "password",
        ],
    ) {
        hints.push(RoleHint {
            weight: 7,
            tokens: vec![
                "auth",
                "authentication",
                "oauth2",
                "jsonwebtoken",
                "argon2",
                "paseto",
                "session",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            queries: vec![
                "oauth2",
                "jsonwebtoken",
                "argon2",
                "authentication",
                "session",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            preferred_names: vec!["oauth2", "jsonwebtoken", "argon2", "pasetors"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        });
    }

    hints
}

fn has_any_token(query_tokens: &[String], values: &[&str]) -> bool {
    query_tokens
        .iter()
        .any(|token| values.iter().any(|value| token == value))
}
