use std::collections::HashSet;

use crate::models::tool::{ToolArguments, ToolCall, ToolPlan};

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

    if contains_any(&normalized, &["hello", "hi", "hey", "say hello"]) {
        return Some("Hello! Ask me about Rust backend tools, repos, or trade-offs.".to_string());
    }

    if contains_any(&normalized, &["how are you"]) {
        return Some(
            "I'm ready to help. Ask me to compare Rust backend tools or find active repositories."
                .to_string(),
        );
    }

    if contains_any(&normalized, &["tell me a joke"]) {
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
    apply_intent_routing(user_message, &normalized, &mut plan);
    rewrite_tool_queries(user_message, &mut plan);
    debug_tools(&mut plan);

    plan.need_tools = !plan.tools.is_empty();
    plan
}

fn normalize(input: &str) -> String {
    input.to_lowercase()
}

fn is_smalltalk(message: &str) -> bool {
    let smalltalk_patterns = [
        "say hello",
        "hello",
        "hi",
        "hey",
        "how are you",
        "tell me a joke",
    ];

    smalltalk_patterns
        .iter()
        .any(|pattern| message.contains(pattern))
}

fn contains_any(message: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| message.contains(pattern))
}

fn apply_intent_routing(user_message: &str, normalized: &str, plan: &mut ToolPlan) {
    if is_compare_only_request(normalized) {
        plan.tools.retain(|tool| tool.name != "github_search");
        ensure_tool(plan, "local_knowledge_search", user_message);
        return;
    }

    if should_use_both_sources(normalized) {
        ensure_tool(plan, "local_knowledge_search", user_message);
        ensure_tool(plan, "github_search", user_message);
    }
}

fn is_compare_only_request(message: &str) -> bool {
    is_comparison_request(message) && !has_discovery_intent(message)
}

fn should_use_both_sources(message: &str) -> bool {
    has_discovery_intent(message) && is_framework_selection_request(message)
}

fn is_comparison_request(message: &str) -> bool {
    contains_any(
        message,
        &[
            "compare",
            "comparison",
            "vs",
            "versus",
            "trade-off",
            "tradeoffs",
            "tradeoff",
            "pros and cons",
            "pros/cons",
            "difference between",
        ],
    )
}

fn has_discovery_intent(message: &str) -> bool {
    contains_any(
        message,
        &[
            "find",
            "find me",
            "show",
            "list",
            "search",
            "looking for",
            "look for",
            "pick",
            "choose",
            "recommend",
            "suggest",
            "some repos",
            "repositories",
            "repository",
            "repo",
            "repos",
            "current",
            "latest",
            "active",
            "recent",
        ],
    )
}

fn is_framework_selection_request(message: &str) -> bool {
    contains_any(
        message,
        &[
            "backend",
            "framework",
            "frameworks",
            "tooling",
            "stack",
            "project",
            "new project",
            "service",
            "api",
            "library",
            "libraries",
            "ecosystem",
            "web",
        ],
    )
}

fn ensure_tool(plan: &mut ToolPlan, name: &str, query: &str) {
    if plan.tools.iter().any(|tool| tool.name == name) {
        return;
    }

    plan.tools.push(tool_call(name, query));
}

fn tool_call(name: &str, query: &str) -> ToolCall {
    let query = match name {
        "github_search" => build_github_query(query),
        _ => query.to_string(),
    };

    ToolCall {
        name: name.to_string(),
        arguments: ToolArguments { query },
    }
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
        }
    }
}

fn retain_supported_tools(plan: &mut ToolPlan) {
    plan.tools.retain(|tool| {
        matches!(
            tool.name.as_str(),
            "github_search" | "local_knowledge_search"
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

fn preferred_github_terms(input: &str) -> Vec<String> {
    let normalized = normalize(input);
    let ordered_terms = [
        "rust",
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
        "postgres",
        "redis",
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

fn debug_tools(plan: &mut ToolPlan) {
    let mut seen = HashSet::new();

    plan.tools.retain(|tool| {
        let key = format!("{}::{}", tool.name, tool.arguments.query);
        seen.insert(key)
    });
}
