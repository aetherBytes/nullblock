pub mod backtest;
pub mod social_monitor;
pub mod strategy_extract;
pub mod url_ingest;
pub mod web_client;
pub mod web_search;

pub use backtest::{BacktestConfig, BacktestEngine, BacktestResult};
pub use social_monitor::{MonitoredSource, SocialAlert, SocialMonitor, SourceType};
pub use strategy_extract::{ExtractedStrategy, StrategyConfidence, StrategyExtractor};
pub use url_ingest::{ContentType, IngestResult, UrlIngester};
pub use web_client::{ExtractMode, WebClient, WebContentType, WebFetchResult};
pub use web_search::{SearchResponse, SearchResult, SerperClient};
