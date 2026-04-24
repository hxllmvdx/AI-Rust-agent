use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CratesSearchResponse {
    pub crates: Vec<CrateSearchItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrateSearchItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub downloads: u64,
    pub max_version: String,
    pub categories: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrateDetailResponse {
    pub categories: Vec<CrateCategory>,
    pub keywords: Vec<CrateKeyword>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrateCategory {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrateKeyword {
    pub id: String,
    pub keyword: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CratesSearchResult {
    pub name: String,
    pub description: Option<String>,
    pub downloads: u64,
    pub latest_version: String,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
}
