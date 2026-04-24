use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    pub id: String,
    pub title: String,
    pub category: String,
    pub tags: Vec<String>,
    pub summary: String,
    pub use_cases: Vec<String>,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub related: Vec<String>,
    pub source: String,
    pub collected_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeSearchResult {
    pub id: String,
    pub title: String,
    pub score: u32,
    pub summary: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}
