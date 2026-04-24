use std::collections::HashSet;

use crate::models::tool::ToolPlan;

pub fn apply_tool_policy(user_message: &str, mut plan: ToolPlan) -> ToolPlan {
    let normalized = normalize(user_message);

    if is_smalltalk(&normalized) {
        plan.tools.clear();
        plan.need_tools = false;
        return plan;
    }

    if !should_allow_github(&normalized) {
        plan.tools.retain(|tool| tool.name != "github_search");
    }

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

fn should_allow_github(message: &str) -> bool {
    let github_keywords = [
        "github",
        "repo",
        "repos",
        "repository",
        "repositories",
        "stars",
        "star",
        "updated",
        "latest",
        "current",
        "active",
        "activity",
        "recent",
    ];

    github_keywords.iter().any(|kw| message.contains(kw))
}

fn debug_tools(plan: &mut ToolPlan) {
    let mut seen = HashSet::new();

    plan.tools.retain(|tool| {
        let key = format!("{}::{}", tool.name, tool.arguments.query);
        seen.insert(key)
    });
}
