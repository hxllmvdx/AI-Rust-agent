use std::collections::{HashMap, HashSet};

use crate::models::tool::ToolPlan;

pub fn fast_path_plan(user_message: &str) -> Option<ToolPlan> {
    let normalized = normalize(user_message);

    if is_smalltalk(&normalized) {
        return Some(ToolPlan {
            need_tools: false,
            tools: Vec::new(),
        });
    }

    None
}

pub fn fast_path_response(user_message: &str) -> Option<String> {
    let normalized = normalize(user_message);
    let tokenized = tokenize(&normalized);

    if is_pure_greeting(&tokenized) || contains_phrase(&normalized, "say hello") {
        return Some("Hello! Ask me about Rust backend tools, repos, or trade-offs.".to_string());
    }

    if contains_phrase(&normalized, "how are you")
        && !contains_technical_intent(&tokenized, &normalized)
    {
        return Some(
            "I'm ready to help. Ask me to compare Rust backend tools or find active repositories."
                .to_string(),
        );
    }

    if contains_phrase(&normalized, "tell me a joke")
        && !contains_technical_intent(&tokenized, &normalized)
    {
        return Some(
            "Rust joke: fearless concurrency is great until your TODO list starts racing too."
                .to_string(),
        );
    }

    None
}

pub fn apply_tool_policy(user_message: &str, mut plan: ToolPlan) -> ToolPlan {
    let normalized = normalize(user_message);

    if is_smalltalk(&normalized) {
        plan.tools.clear();
        plan.need_tools = false;
        return plan;
    }

    retain_supported_tools(&mut plan);
    rewrite_tool_queries(user_message, &mut plan);
    debug_tools(&mut plan);

    plan.need_tools = !plan.tools.is_empty();
    plan
}

fn normalize(input: &str) -> String {
    input.to_lowercase()
}

fn is_smalltalk(message: &str) -> bool {
    let normalized = normalize(message);
    let tokenized = tokenize(&normalized);

    if contains_technical_intent(&tokenized, &normalized) {
        return false;
    }

    let token_count = tokenized.len();
    let has_smalltalk_phrase = contains_phrase(&normalized, "say hello")
        || contains_phrase(&normalized, "how are you")
        || contains_phrase(&normalized, "tell me a joke");
    let all_tokens_smalltalk = token_count > 0
        && tokenized.iter().all(|token| {
            matches!(
                token.as_str(),
                "hello"
                    | "hi"
                    | "hey"
                    | "how"
                    | "are"
                    | "you"
                    | "tell"
                    | "me"
                    | "a"
                    | "joke"
                    | "say"
            )
        });

    (has_smalltalk_phrase && token_count <= 8)
        || (all_tokens_smalltalk && token_count <= 4)
        || (token_count == 1
            && tokenized
                .iter()
                .any(|token| matches!(token.as_str(), "hello" | "hi" | "hey")))
}

fn contains_phrase(message: &str, phrase: &str) -> bool {
    message.contains(phrase)
}

fn tokenize(message: &str) -> Vec<String> {
    message
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_string())
        .collect()
}

fn contains_technical_intent(tokens: &[String], normalized: &str) -> bool {
    let technical_tokens = [
        "rust",
        "backend",
        "framework",
        "frameworks",
        "database",
        "databases",
        "routing",
        "router",
        "api",
        "web",
        "server",
        "service",
        "services",
        "tool",
        "tools",
        "repo",
        "repos",
        "repository",
        "repositories",
        "github",
        "sql",
        "orm",
        "postgres",
        "mysql",
        "sqlite",
        "actix",
        "axum",
        "warp",
        "rocket",
        "tokio",
        "observability",
        "tracing",
        "auth",
        "authentication",
        "recommend",
        "advice",
        "find",
        "learn",
        "code",
        "concurrent",
        "fast",
    ];

    tokens
        .iter()
        .any(|token| technical_tokens.contains(&token.as_str()))
        || contains_phrase(normalized, "high load")
        || contains_phrase(normalized, "high-load")
        || contains_phrase(normalized, "write backend code")
        || contains_phrase(normalized, "find me")
        || contains_phrase(normalized, "recommend me")
}

fn is_pure_greeting(tokens: &[String]) -> bool {
    let token_count = tokens.len();
    token_count > 0
        && token_count <= 3
        && tokens
            .iter()
            .all(|token| matches!(token.as_str(), "hello" | "hi" | "hey"))
}

fn rewrite_tool_queries(user_message: &str, plan: &mut ToolPlan) {
    for tool in &mut plan.tools {
        if tool.name == "github_search" {
            let source = if tool.arguments.query.trim().is_empty() {
                user_message
            } else {
                &tool.arguments.query
            };
            tool.arguments.query = build_github_query(source);
        } else if tool.name == "crates_search" {
            let source = if tool.arguments.query.trim().is_empty() {
                user_message
            } else {
                &tool.arguments.query
            };
            tool.arguments.query = build_crates_query(source);
        }
    }
}

fn retain_supported_tools(plan: &mut ToolPlan) {
    plan.tools.retain(|tool| {
        matches!(
            tool.name.as_str(),
            "github_search" | "local_knowledge_search" | "crates_search"
        )
    });
}

fn build_github_query(input: &str) -> String {
    let preferred = preferred_github_terms(input);
    if !preferred.is_empty() {
        return preferred.join(" ");
    }

    let fallback = generic_github_terms(input);
    if !fallback.is_empty() {
        return fallback.join(" ");
    }

    "rust backend".to_string()
}

fn build_crates_query(input: &str) -> String {
    let mut terms = preferred_crates_terms(input);
    expand_crates_terms(input, &mut terms);
    dedup_terms(&mut terms);
    if !terms.is_empty() {
        terms.truncate(8);
        return terms.join(" ");
    }

    let mut fallback = generic_crates_terms(input);
    expand_crates_terms(input, &mut fallback);
    dedup_terms(&mut fallback);
    if !fallback.is_empty() {
        fallback.truncate(8);
        return fallback.join(" ");
    }

    "rust library".to_string()
}

fn preferred_github_terms(input: &str) -> Vec<String> {
    let normalized = normalize(input);
    let ordered_terms = [
        "rust",
        "sql",
        "backend",
        "framework",
        "frameworks",
        "web",
        "api",
        "server",
        "service",
        "microservice",
        "tokio",
        "axum",
        "actix",
        "warp",
        "rocket",
        "observability",
        "tracing",
        "telemetry",
        "metrics",
        "logging",
        "authentication",
        "auth",
        "database",
        "orm",
        "sqlx",
        "diesel",
        "seaorm",
        "sea-orm",
        "postgres",
        "redis",
        "mysql",
        "sqlite",
        "postgresql",
        "graphql",
        "grpc",
        "tooling",
        "library",
        "libraries",
    ];

    let mut selected = Vec::new();
    for term in ordered_terms {
        if normalized.contains(term) && !selected.iter().any(|item| item == term) {
            selected.push(term.to_string());
        }

        if selected.len() >= 5 {
            break;
        }
    }

    selected
}

fn preferred_crates_terms(input: &str) -> Vec<String> {
    let normalized = normalize(input);
    let ordered_terms = [
        "rust",
        "config",
        "configuration",
        "env",
        "environment",
        "secret",
        "secrets",
        "cli",
        "command",
        "parsing",
        "terminal",
        "tracing",
        "metrics",
        "logging",
        "observability",
        "correlation",
        "database",
        "databases",
        "sql",
        "orm",
        "postgres",
        "mysql",
        "sqlite",
        "auth",
        "authentication",
        "cache",
        "queue",
        "serialization",
        "json",
        "toml",
        "yaml",
        "framework",
        "frameworks",
        "library",
        "libraries",
    ];

    let mut selected = Vec::new();
    for term in ordered_terms {
        if normalized.contains(term) && !selected.iter().any(|item| item == term) {
            selected.push(term.to_string());
        }

        if selected.len() >= 6 {
            break;
        }
    }

    selected
}

fn expand_crates_terms(input: &str, terms: &mut Vec<String>) {
    let normalized = normalize(input);

    if contains_any_phrase(
        &normalized,
        &[
            "config",
            "configuration",
            "env",
            "environment",
            "secret",
            "secrets",
            "settings",
        ],
    ) {
        terms.extend(
            [
                "config",
                "configuration",
                "figment",
                "confique",
                "dotenvy",
                "secrecy",
            ]
            .into_iter()
            .map(str::to_string),
        );
    }

    if contains_any_phrase(
        &normalized,
        &[
            "metrics",
            "metric",
            "tracing",
            "trace",
            "log correlation",
            "correlation",
            "logging",
            "observability",
            "telemetry",
        ],
    ) {
        terms.extend(
            [
                "metrics",
                "tracing",
                "tracing-subscriber",
                "opentelemetry",
                "tower-http",
                "axum-tracing-opentelemetry",
            ]
            .into_iter()
            .map(str::to_string),
        );
    }

    if contains_any_phrase(
        &normalized,
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
        terms.extend(
            [
                "sqlx", "diesel", "sea-orm", "seaorm", "postgres", "sqlite", "mysql",
            ]
            .into_iter()
            .map(str::to_string),
        );
    }

    if contains_any_phrase(
        &normalized,
        &[
            "cli",
            "command line",
            "argument parsing",
            "terminal",
            "console",
        ],
    ) {
        terms.extend(
            ["clap", "argh", "bpaf", "dialoguer", "ratatui"]
                .into_iter()
                .map(str::to_string),
        );
    }

    if contains_any_phrase(&normalized, &["auth", "authentication", "jwt", "session"]) {
        terms.extend(
            ["jsonwebtoken", "axum-login", "oauth2", "argon2", "pasetors"]
                .into_iter()
                .map(str::to_string),
        );
    }
}

fn generic_github_terms(input: &str) -> Vec<String> {
    let stop_words = [
        "a",
        "an",
        "and",
        "are",
        "be",
        "best",
        "build",
        "building",
        "care",
        "compare",
        "current",
        "find",
        "for",
        "from",
        "help",
        "i",
        "im",
        "in",
        "include",
        "looking",
        "maybe",
        "me",
        "my",
        "new",
        "of",
        "project",
        "recommend",
        "repositories",
        "repository",
        "search",
        "show",
        "some",
        "that",
        "the",
        "their",
        "them",
        "to",
        "use",
        "what",
        "with",
        "would",
        "you",
    ];

    let mut selected = Vec::new();
    for token in normalize(input)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| !token.is_empty())
    {
        if token.len() < 3 {
            continue;
        }

        if stop_words.contains(&token) {
            continue;
        }

        if !selected.iter().any(|item| item == token) {
            selected.push(token.to_string());
        }

        if selected.len() >= 5 {
            break;
        }
    }

    selected
}

fn generic_crates_terms(input: &str) -> Vec<String> {
    let stop_words = [
        "a",
        "about",
        "advice",
        "also",
        "an",
        "and",
        "are",
        "best",
        "building",
        "can",
        "choose",
        "concrete",
        "evaluate",
        "for",
        "from",
        "give",
        "help",
        "i",
        "in",
        "libraries",
        "library",
        "me",
        "modern",
        "need",
        "pick",
        "please",
        "practical",
        "production",
        "recommend",
        "related",
        "service",
        "should",
        "some",
        "stack",
        "the",
        "things",
        "use",
        "useful",
        "what",
        "which",
        "with",
    ];

    let mut selected = Vec::new();
    for token in tokenize(&normalize(input)) {
        if token.len() < 3 {
            continue;
        }
        if stop_words.contains(&token.as_str()) {
            continue;
        }
        if !selected.iter().any(|item| item == &token) {
            selected.push(token);
        }
        if selected.len() >= 6 {
            break;
        }
    }

    if !selected.iter().any(|item| item == "rust") {
        selected.insert(0, "rust".to_string());
    }

    selected
}

fn contains_any_phrase(message: &str, phrases: &[&str]) -> bool {
    phrases
        .iter()
        .any(|phrase| contains_phrase(message, phrase))
}

fn dedup_terms(terms: &mut Vec<String>) {
    let mut seen = HashSet::new();
    terms.retain(|term| seen.insert(term.clone()));
}

fn debug_tools(plan: &mut ToolPlan) {
    let mut seen_exact = HashSet::new();
    plan.tools.retain(|tool| {
        let key = format!("{}::{}", tool.name, tool.arguments.query);
        seen_exact.insert(key)
    });

    let mut best_by_tool: HashMap<String, usize> = HashMap::new();
    for (index, tool) in plan.tools.iter().enumerate() {
        best_by_tool
            .entry(tool.name.clone())
            .and_modify(|best_index| {
                if is_better_query(
                    &tool.arguments.query,
                    &plan.tools[*best_index].arguments.query,
                ) {
                    *best_index = index;
                }
            })
            .or_insert(index);
    }

    plan.tools = plan
        .tools
        .iter()
        .enumerate()
        .filter(|(index, tool)| best_by_tool.get(&tool.name) == Some(index))
        .map(|(_, tool)| tool.clone())
        .collect();
}

fn is_better_query(candidate: &str, current: &str) -> bool {
    let candidate_tokens = tokenize(&normalize(candidate));
    let current_tokens = tokenize(&normalize(current));

    if candidate_tokens.len() != current_tokens.len() {
        return candidate_tokens.len() > current_tokens.len();
    }

    candidate.len() > current.len()
}
