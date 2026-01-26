use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub position: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub query: String,
    pub search_type: String,
    pub results: Vec<SearchResult>,
    pub total_results: u32,
    pub search_time_ms: u64,
}

#[derive(Debug, Deserialize)]
struct SerperResponse {
    #[serde(default)]
    organic: Vec<SerperOrganic>,
    #[serde(default)]
    news: Vec<SerperNews>,
    #[serde(rename = "searchParameters")]
    search_parameters: Option<SerperSearchParams>,
}

#[derive(Debug, Deserialize)]
struct SerperOrganic {
    title: String,
    link: String,
    snippet: Option<String>,
    position: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct SerperNews {
    title: String,
    link: String,
    snippet: Option<String>,
    date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SerperSearchParams {
    q: Option<String>,
}

pub struct SerperClient {
    client: Client,
    api_url: String,
    api_key: Option<String>,
}

impl SerperClient {
    pub fn new(api_url: String, api_key: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            api_url,
            api_key,
        }
    }

    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }

    pub async fn search(
        &self,
        query: &str,
        num_results: u32,
        search_type: &str,
        time_range: Option<&str>,
    ) -> Result<SearchResponse, String> {
        let api_key = match &self.api_key {
            Some(key) => key,
            None => {
                return Err(
                    "Serper API key not configured. Set SERPER_API_KEY environment variable. \
                    You can still use web_fetch to fetch content from URLs directly."
                        .to_string(),
                )
            }
        };

        let start = std::time::Instant::now();
        let endpoint = match search_type {
            "news" => format!("{}/news", self.api_url),
            _ => format!("{}/search", self.api_url),
        };

        let mut body = serde_json::json!({
            "q": query,
            "num": num_results.min(10),
        });

        if let Some(range) = time_range {
            let tbs = match range {
                "day" => "qdr:d",
                "week" => "qdr:w",
                "month" => "qdr:m",
                "year" => "qdr:y",
                _ => "qdr:w",
            };
            body["tbs"] = serde_json::json!(tbs);
        }

        debug!("Serper search: {} type={}", query, search_type);

        match self
            .client
            .post(&endpoint)
            .header("X-API-KEY", api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<SerperResponse>().await {
                        Ok(serper_resp) => {
                            let results: Vec<SearchResult> = if search_type == "news" {
                                serper_resp
                                    .news
                                    .into_iter()
                                    .enumerate()
                                    .map(|(i, item)| SearchResult {
                                        title: item.title,
                                        url: item.link,
                                        snippet: item.snippet.unwrap_or_else(|| {
                                            item.date.unwrap_or_default()
                                        }),
                                        position: (i + 1) as u32,
                                    })
                                    .collect()
                            } else {
                                serper_resp
                                    .organic
                                    .into_iter()
                                    .map(|item| SearchResult {
                                        title: item.title,
                                        url: item.link,
                                        snippet: item.snippet.unwrap_or_default(),
                                        position: item.position.unwrap_or(0),
                                    })
                                    .collect()
                            };

                            let elapsed = start.elapsed().as_millis() as u64;
                            info!(
                                "Serper search completed: {} results in {}ms",
                                results.len(),
                                elapsed
                            );

                            Ok(SearchResponse {
                                query: query.to_string(),
                                search_type: search_type.to_string(),
                                total_results: results.len() as u32,
                                results,
                                search_time_ms: elapsed,
                            })
                        }
                        Err(e) => {
                            error!("Failed to parse Serper response: {}", e);
                            Err(format!("Failed to parse search response: {}", e))
                        }
                    }
                } else {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    error!("Serper API error {}: {}", status, body);
                    Err(format!("Serper API error {}: {}", status, body))
                }
            }
            Err(e) => {
                error!("Serper request failed: {}", e);
                Err(format!("Search request failed: {}", e))
            }
        }
    }
}

impl Default for SerperClient {
    fn default() -> Self {
        Self::new(
            "https://google.serper.dev".to_string(),
            std::env::var("SERPER_API_KEY").ok(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serper_client_not_configured() {
        let client = SerperClient::new("https://google.serper.dev".to_string(), None);
        assert!(!client.is_configured());
    }

    #[test]
    fn test_serper_client_configured() {
        let client =
            SerperClient::new("https://google.serper.dev".to_string(), Some("test-key".to_string()));
        assert!(client.is_configured());
    }
}
