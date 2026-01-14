use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Tweet,
    Thread,
    Article,
    Documentation,
    Forum,
    Unknown,
}

impl ContentType {
    pub fn from_url(url: &str) -> Self {
        let url_lower = url.to_lowercase();

        if url_lower.contains("twitter.com") || url_lower.contains("x.com") {
            if url_lower.contains("/status/") {
                ContentType::Tweet
            } else {
                ContentType::Thread
            }
        } else if url_lower.contains("medium.com")
            || url_lower.contains("substack.com")
            || url_lower.contains("mirror.xyz") {
            ContentType::Article
        } else if url_lower.contains("docs.") || url_lower.contains("/docs/") {
            ContentType::Documentation
        } else if url_lower.contains("reddit.com") || url_lower.contains("discord.com") {
            ContentType::Forum
        } else {
            ContentType::Unknown
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestResult {
    pub id: Uuid,
    pub url: String,
    pub content_type: ContentType,
    pub title: Option<String>,
    pub author: Option<String>,
    pub raw_content: String,
    pub cleaned_content: String,
    pub extracted_tokens: Vec<String>,
    pub extracted_addresses: Vec<String>,
    pub extracted_numbers: Vec<ExtractedNumber>,
    pub metadata: serde_json::Value,
    pub ingested_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedNumber {
    pub value: f64,
    pub context: String,
    pub unit: Option<String>,
}

pub struct UrlIngester {
    client: Client,
    user_agent: String,
}

impl UrlIngester {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            user_agent: "Mozilla/5.0 (compatible; ArbFarm/1.0; +https://nullblock.io)".to_string(),
        }
    }

    pub async fn ingest(&self, url: &str) -> AppResult<IngestResult> {
        let content_type = ContentType::from_url(url);

        let response = self.client
            .get(url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await?;

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        let title = self.extract_title(&document);
        let author = self.extract_author(&document, &content_type);
        let raw_content = self.extract_content(&document, &content_type);
        let cleaned_content = self.clean_content(&raw_content);

        let extracted_tokens = self.extract_token_mentions(&cleaned_content);
        let extracted_addresses = self.extract_solana_addresses(&cleaned_content);
        let extracted_numbers = self.extract_numbers(&cleaned_content);

        Ok(IngestResult {
            id: Uuid::new_v4(),
            url: url.to_string(),
            content_type,
            title,
            author,
            raw_content,
            cleaned_content,
            extracted_tokens,
            extracted_addresses,
            extracted_numbers,
            metadata: serde_json::json!({}),
            ingested_at: Utc::now(),
        })
    }

    fn extract_title(&self, document: &Html) -> Option<String> {
        let selectors = [
            "meta[property='og:title']",
            "meta[name='twitter:title']",
            "title",
            "h1",
        ];

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    if selector_str.starts_with("meta") {
                        if let Some(content) = element.value().attr("content") {
                            return Some(content.to_string());
                        }
                    } else {
                        let text: String = element.text().collect();
                        if !text.trim().is_empty() {
                            return Some(text.trim().to_string());
                        }
                    }
                }
            }
        }

        None
    }

    fn extract_author(&self, document: &Html, content_type: &ContentType) -> Option<String> {
        let selectors = match content_type {
            ContentType::Tweet | ContentType::Thread => vec![
                "meta[property='og:title']",
                "[data-testid='User-Name']",
            ],
            ContentType::Article => vec![
                "meta[name='author']",
                "meta[property='article:author']",
                "[rel='author']",
                ".author",
            ],
            _ => vec![
                "meta[name='author']",
                ".author",
            ],
        };

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    if selector_str.starts_with("meta") {
                        if let Some(content) = element.value().attr("content") {
                            let author = self.extract_twitter_handle(content);
                            return Some(author);
                        }
                    } else {
                        let text: String = element.text().collect();
                        if !text.trim().is_empty() {
                            return Some(text.trim().to_string());
                        }
                    }
                }
            }
        }

        None
    }

    fn extract_twitter_handle(&self, text: &str) -> String {
        if text.contains("(@") {
            if let Some(start) = text.find("(@") {
                if let Some(end) = text[start..].find(')') {
                    return text[start + 1..start + end].to_string();
                }
            }
        }
        text.to_string()
    }

    fn extract_content(&self, document: &Html, content_type: &ContentType) -> String {
        let selectors = match content_type {
            ContentType::Tweet | ContentType::Thread => vec![
                "[data-testid='tweetText']",
                "article",
                "meta[property='og:description']",
            ],
            ContentType::Article => vec![
                "article",
                ".post-content",
                ".article-content",
                "main",
            ],
            ContentType::Documentation => vec![
                "main",
                ".content",
                "article",
            ],
            ContentType::Forum => vec![
                ".post-content",
                ".comment",
                "main",
            ],
            ContentType::Unknown => vec![
                "main",
                "article",
                "body",
            ],
        };

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                let elements: Vec<_> = document.select(&selector).collect();
                if !elements.is_empty() {
                    let content: String = elements
                        .iter()
                        .map(|e| {
                            if selector_str.starts_with("meta") {
                                e.value().attr("content").unwrap_or("").to_string()
                            } else {
                                e.text().collect::<String>()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n\n");

                    if !content.trim().is_empty() {
                        return content;
                    }
                }
            }
        }

        document.root_element().text().collect()
    }

    fn clean_content(&self, raw: &str) -> String {
        let mut cleaned = raw.to_string();

        cleaned = cleaned
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty()
                    && trimmed.len() > 2
                    && !trimmed.starts_with("//")
                    && !trimmed.starts_with("/*")
            })
            .collect::<Vec<_>>()
            .join("\n");

        let patterns_to_remove = [
            "Subscribe",
            "Sign up",
            "Log in",
            "Cookie",
            "Privacy Policy",
            "Terms of Service",
        ];

        for pattern in patterns_to_remove {
            cleaned = cleaned.replace(pattern, "");
        }

        cleaned
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn extract_token_mentions(&self, content: &str) -> Vec<String> {
        let mut tokens = Vec::new();

        let words: Vec<&str> = content.split_whitespace().collect();
        for word in words {
            if word.starts_with('$') && word.len() > 1 && word.len() < 20 {
                let token = word
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_uppercase();
                if !token.is_empty() && !tokens.contains(&token) {
                    tokens.push(token);
                }
            }
        }

        let common_tokens = ["SOL", "USDC", "USDT", "ETH", "BTC", "BONK", "JUP", "RAY"];
        for token in common_tokens {
            if content.to_uppercase().contains(token) && !tokens.contains(&token.to_string()) {
                tokens.push(token.to_string());
            }
        }

        tokens
    }

    fn extract_solana_addresses(&self, content: &str) -> Vec<String> {
        let mut addresses = Vec::new();

        let words: Vec<&str> = content.split_whitespace().collect();
        for word in words {
            let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric());
            if cleaned.len() >= 32 && cleaned.len() <= 44 {
                if cleaned.chars().all(|c| c.is_alphanumeric()) {
                    if !addresses.contains(&cleaned.to_string()) {
                        addresses.push(cleaned.to_string());
                    }
                }
            }
        }

        addresses
    }

    fn extract_numbers(&self, content: &str) -> Vec<ExtractedNumber> {
        let mut numbers = Vec::new();

        let patterns = [
            (r"(\d+(?:\.\d+)?)\s*%", "percentage"),
            (r"(\d+(?:\.\d+)?)\s*(?:SOL|sol)", "sol_amount"),
            (r"\$(\d+(?:,\d{3})*(?:\.\d+)?)", "usd_amount"),
            (r"(\d+(?:\.\d+)?)\s*x", "multiplier"),
            (r"(\d+)(?:bps|BPS)", "basis_points"),
        ];

        for (pattern, context) in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                for cap in re.captures_iter(content) {
                    if let Some(num_match) = cap.get(1) {
                        let num_str = num_match.as_str().replace(',', "");
                        if let Ok(value) = num_str.parse::<f64>() {
                            numbers.push(ExtractedNumber {
                                value,
                                context: context.to_string(),
                                unit: Some(context.to_string()),
                            });
                        }
                    }
                }
            }
        }

        numbers
    }

    pub async fn ingest_tweet(&self, tweet_url: &str) -> AppResult<IngestResult> {
        let mut result = self.ingest(tweet_url).await?;
        result.content_type = ContentType::Tweet;

        if let Some(status_id) = self.extract_tweet_id(tweet_url) {
            result.metadata = serde_json::json!({
                "tweet_id": status_id,
                "platform": "twitter"
            });
        }

        Ok(result)
    }

    fn extract_tweet_id(&self, url: &str) -> Option<String> {
        if let Some(pos) = url.find("/status/") {
            let after_status = &url[pos + 8..];
            let id: String = after_status
                .chars()
                .take_while(|c| c.is_numeric())
                .collect();
            if !id.is_empty() {
                return Some(id);
            }
        }
        None
    }
}

impl Default for UrlIngester {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_detection() {
        assert!(matches!(
            ContentType::from_url("https://twitter.com/user/status/123"),
            ContentType::Tweet
        ));
        assert!(matches!(
            ContentType::from_url("https://x.com/user/status/123"),
            ContentType::Tweet
        ));
        assert!(matches!(
            ContentType::from_url("https://medium.com/article"),
            ContentType::Article
        ));
    }

    #[test]
    fn test_extract_tokens() {
        let ingester = UrlIngester::new();
        let content = "Buy $BONK and $SOL for gains! Also check $WIF";
        let tokens = ingester.extract_token_mentions(content);

        assert!(tokens.contains(&"BONK".to_string()));
        assert!(tokens.contains(&"SOL".to_string()));
        assert!(tokens.contains(&"WIF".to_string()));
    }

    #[test]
    fn test_extract_numbers() {
        let ingester = UrlIngester::new();
        let content = "Target 50% profit with 2 SOL position. Entry at $0.001";
        let numbers = ingester.extract_numbers(content);

        assert!(!numbers.is_empty());
    }
}
