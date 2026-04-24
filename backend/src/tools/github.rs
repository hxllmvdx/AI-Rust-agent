use reqwest::Client;

use crate::{
    error::BackendError,
    models::github::{GitHubSearchResponse, GitHubSearchResult},
};

#[derive(Clone)]
pub struct GitHubTool {
    http: Client,
    token: Option<String>,
}

impl GitHubTool {
    pub fn new(token: Option<String>) -> Self {
        Self {
            http: Client::new(),
            token,
        }
    }

    pub async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<GitHubSearchResult>, BackendError> {
        let url = "https://api.github.com/search/repositories";

        let mut request = self
            .http
            .get(url)
            .query(&[
                ("q", query),
                ("sort", "stars"),
                ("order", "desc"),
                ("per_page", &limit.to_string()),
            ])
            .header("User-Agent", "ai-rust-agent");

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await?
            .error_for_status()?
            .json::<GitHubSearchResponse>()
            .await?;

        let results = response
            .items
            .into_iter()
            .map(|item| GitHubSearchResult {
                full_name: item.full_name,
                description: item.description,
                html_url: item.html_url,
                language: item.language,
                stargazers_count: item.stargazers_count,
                updated_at: item.updated_at,
            })
            .collect();

        Ok(results)
    }
}
