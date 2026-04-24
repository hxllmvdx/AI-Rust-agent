use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubSearchResponse {
    pub items: Vec<GitHubRepoItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRepoItem {
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub language: Option<String>,
    pub stargazers_count: u64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GitHubSearchResult {
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub language: Option<String>,
    pub stargazers_count: u64,
    pub updated_at: String,
}
