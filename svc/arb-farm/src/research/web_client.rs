use chrono::{DateTime, Utc};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchResult {
    pub id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub content: String,
    pub content_type: WebContentType,
    pub word_count: usize,
    pub extracted_tokens: Vec<String>,
    pub extracted_addresses: Vec<String>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebContentType {
    Tweet,
    Thread,
    Article,
    Documentation,
    Forum,
    News,
    Blog,
    Unknown,
}

impl WebContentType {
    pub fn from_url(url: &str) -> Self {
        let url_lower = url.to_lowercase();

        if url_lower.contains("twitter.com") || url_lower.contains("x.com") {
            if url_lower.contains("/status/") {
                WebContentType::Tweet
            } else {
                WebContentType::Thread
            }
        } else if url_lower.contains("medium.com")
            || url_lower.contains("substack.com")
            || url_lower.contains("mirror.xyz")
        {
            WebContentType::Article
        } else if url_lower.contains("docs.") || url_lower.contains("/docs/") {
            WebContentType::Documentation
        } else if url_lower.contains("reddit.com") || url_lower.contains("discord.com") {
            WebContentType::Forum
        } else if url_lower.contains("news")
            || url_lower.contains("coindesk")
            || url_lower.contains("cointelegraph")
        {
            WebContentType::News
        } else if url_lower.contains("blog") {
            WebContentType::Blog
        } else {
            WebContentType::Unknown
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractMode {
    Full,
    Article,
    Summary,
}

impl Default for ExtractMode {
    fn default() -> Self {
        ExtractMode::Article
    }
}

pub struct WebClient {
    client: Client,
    user_agent: String,
}

impl WebClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .redirect(reqwest::redirect::Policy::limited(5))
                .build()
                .unwrap_or_default(),
            user_agent: "Mozilla/5.0 (compatible; ArbFarm/1.0; +https://nullblock.io)".to_string(),
        }
    }

    pub async fn fetch(
        &self,
        url: &str,
        extract_mode: ExtractMode,
        max_length: usize,
    ) -> Result<WebFetchResult, String> {
        let start = std::time::Instant::now();
        debug!("Fetching URL: {} mode={:?}", url, extract_mode);

        let response = self
            .client
            .get(url)
            .header("User-Agent", &self.user_agent)
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .send()
            .await
            .map_err(|e| format!("Failed to fetch URL: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let html = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let document = Html::parse_document(&html);
        let content_type = WebContentType::from_url(url);

        let title = self.extract_title(&document);
        let author = self.extract_author(&document, &content_type);
        let raw_content = self.extract_content(&document, &content_type);

        let content = match extract_mode {
            ExtractMode::Full => self.clean_content(&raw_content),
            ExtractMode::Article => self.extract_article_content(&raw_content, max_length),
            ExtractMode::Summary => self.create_summary(&raw_content, max_length / 2),
        };

        let content = if content.len() > max_length {
            content[..max_length].to_string()
        } else {
            content
        };

        let word_count = content.split_whitespace().count();
        let extracted_tokens = self.extract_token_mentions(&content);
        let extracted_addresses = self.extract_solana_addresses(&content);

        let elapsed = start.elapsed().as_millis();
        info!("Fetched {} ({} words) in {}ms", url, word_count, elapsed);

        Ok(WebFetchResult {
            id: Uuid::new_v4(),
            url: url.to_string(),
            title,
            author,
            content,
            content_type,
            word_count,
            extracted_tokens,
            extracted_addresses,
            fetched_at: Utc::now(),
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

    fn extract_author(&self, document: &Html, content_type: &WebContentType) -> Option<String> {
        let selectors = match content_type {
            WebContentType::Tweet | WebContentType::Thread => {
                vec!["meta[property='og:title']", "[data-testid='User-Name']"]
            }
            WebContentType::Article | WebContentType::Blog => vec![
                "meta[name='author']",
                "meta[property='article:author']",
                "[rel='author']",
                ".author",
                ".byline",
            ],
            _ => vec!["meta[name='author']", ".author"],
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

    fn extract_content(&self, document: &Html, content_type: &WebContentType) -> String {
        let selectors = match content_type {
            WebContentType::Tweet | WebContentType::Thread => vec![
                "[data-testid='tweetText']",
                "article",
                "meta[property='og:description']",
            ],
            WebContentType::Article | WebContentType::Blog => vec![
                "article",
                ".post-content",
                ".article-content",
                ".entry-content",
                "main",
            ],
            WebContentType::Documentation => vec!["main", ".content", "article", ".documentation"],
            WebContentType::Forum => vec![".post-content", ".comment", "main"],
            WebContentType::News => vec!["article", ".article-body", ".story-body", "main"],
            WebContentType::Unknown => vec!["main", "article", "body"],
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
            "Advertisement",
            "ADVERTISEMENT",
        ];

        for pattern in patterns_to_remove {
            cleaned = cleaned.replace(pattern, "");
        }

        cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn extract_article_content(&self, raw: &str, max_length: usize) -> String {
        let cleaned = self.clean_content(raw);

        let paragraphs: Vec<&str> = cleaned.split("\n\n").filter(|p| p.len() > 50).collect();

        let mut result = String::new();
        for para in paragraphs {
            if result.len() + para.len() > max_length {
                break;
            }
            if !result.is_empty() {
                result.push_str("\n\n");
            }
            result.push_str(para);
        }

        if result.is_empty() {
            cleaned[..cleaned.len().min(max_length)].to_string()
        } else {
            result
        }
    }

    fn create_summary(&self, raw: &str, max_length: usize) -> String {
        let cleaned = self.clean_content(raw);

        let sentences: Vec<&str> = cleaned
            .split(|c| c == '.' || c == '!' || c == '?')
            .filter(|s| s.len() > 20)
            .take(5)
            .collect();

        let summary = sentences.join(". ");
        if summary.len() > max_length {
            format!("{}...", &summary[..max_length - 3])
        } else {
            summary
        }
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

        let common_tokens = [
            "SOL", "USDC", "USDT", "ETH", "BTC", "BONK", "JUP", "RAY", "WIF", "PEPE",
        ];
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
                    if !cleaned.chars().all(|c| c.is_numeric()) {
                        if !addresses.contains(&cleaned.to_string()) {
                            addresses.push(cleaned.to_string());
                        }
                    }
                }
            }
        }

        addresses
    }
}

impl Default for WebClient {
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
            WebContentType::from_url("https://twitter.com/user/status/123"),
            WebContentType::Tweet
        ));
        assert!(matches!(
            WebContentType::from_url("https://x.com/user/status/123"),
            WebContentType::Tweet
        ));
        assert!(matches!(
            WebContentType::from_url("https://medium.com/article"),
            WebContentType::Article
        ));
        assert!(matches!(
            WebContentType::from_url("https://docs.example.com"),
            WebContentType::Documentation
        ));
    }

    #[test]
    fn test_extract_tokens() {
        let client = WebClient::new();
        let content = "Buy $BONK and $SOL for gains! Also check $WIF";
        let tokens = client.extract_token_mentions(content);

        assert!(tokens.contains(&"BONK".to_string()));
        assert!(tokens.contains(&"SOL".to_string()));
        assert!(tokens.contains(&"WIF".to_string()));
    }
}
