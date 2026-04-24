use std::fs;

use crate::{
    error::BackendError,
    models::local_data::{KnowledgeItem, KnowledgeSearchResult},
};

#[derive(Clone)]
pub struct LocalKnowledgeTool {
    items: Vec<KnowledgeItem>,
}

impl LocalKnowledgeTool {
    pub fn load_from_file(path: &str) -> Result<Self, BackendError> {
        let raw = fs::read_to_string(path).map_err(|e| BackendError::Io(e))?;
        let items: Vec<KnowledgeItem> = serde_json::from_str(&raw)?;
        Ok(Self { items })
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<KnowledgeSearchResult> {
        let tokens = normalize_query(query);

        let mut result: Vec<KnowledgeSearchResult> = self
            .items
            .iter()
            .filter_map(|item| {
                let score = score_item(item, &tokens);

                if score == 0 {
                    return None;
                }

                Some(KnowledgeSearchResult {
                    id: item.id.clone(),
                    title: item.title.clone(),
                    score,
                    summary: item.summary.clone(),
                    pros: item.pros.clone(),
                    cons: item.cons.clone(),
                })
            })
            .collect();

        result.sort_by(|a, b| b.score.cmp(&a.score));
        result.truncate(limit);
        result
    }
}

fn normalize_query(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn score_item(item: &KnowledgeItem, tokens: &[String]) -> u32 {
    let mut score = 0;

    let title = item.title.to_lowercase();
    let category = item.category.to_lowercase();
    let summary = item.summary.to_lowercase();

    for token in tokens {
        if title.contains(token) {
            score += 5;
        }

        if category.contains(token) {
            score += 3;
        }

        if summary.contains(token) {
            score += 2;
        }

        if item.tags.iter().any(|x| x.to_lowercase().contains(token)) {
            score += 4;
        }

        if item
            .use_cases
            .iter()
            .any(|x| x.to_lowercase().contains(token))
        {
            score += 3;
        }

        if item.pros.iter().any(|x| x.to_lowercase().contains(token)) {
            score += 1;
        }

        if item.cons.iter().any(|x| x.to_lowercase().contains(token)) {
            score += 1;
        }

        if item
            .related
            .iter()
            .any(|x| x.to_lowercase().contains(token))
        {
            score += 1;
        }
    }

    score
}
