pub mod url_ingest;
pub mod strategy_extract;
pub mod backtest;
pub mod social_monitor;
pub mod web_search;
pub mod web_client;

pub use url_ingest::{UrlIngester, IngestResult, ContentType};
pub use strategy_extract::{StrategyExtractor, ExtractedStrategy, StrategyConfidence};
pub use backtest::{BacktestEngine, BacktestResult, BacktestConfig};
pub use social_monitor::{SocialMonitor, MonitoredSource, SourceType, SocialAlert};
pub use web_search::{SerperClient, SearchResult, SearchResponse};
pub use web_client::{WebClient, WebFetchResult, WebContentType, ExtractMode};
