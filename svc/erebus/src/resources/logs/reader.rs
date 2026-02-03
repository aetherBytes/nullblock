use super::types::{LogCategory, LogEntry, LogLevel};
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use tracing::debug;

pub struct LogReader {
    log_paths: Vec<PathBuf>,
}

impl LogReader {
    pub fn new() -> Self {
        let base_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let log_paths = vec![
            base_path.join("logs/erebus.log"),
            base_path.join("svc/nullblock-agents/logs/nullblock-agents.log"),
            base_path.join("svc/nullblock-protocols/logs/nullblock-protocols.log"),
        ];

        Self { log_paths }
    }

    pub fn read_recent_logs(&self, limit: usize) -> Vec<LogEntry> {
        let mut all_logs = Vec::new();

        for path in &self.log_paths {
            if let Ok(file) = fs::File::open(path) {
                let reader = BufReader::new(file);
                let source = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                for line in reader.lines().map_while(Result::ok) {
                    if let Some(entry) = self.parse_log_line(&line, &source) {
                        all_logs.push(entry);
                    }
                }
            } else {
                debug!("Could not open log file: {:?}", path);
            }
        }

        // Sort by timestamp (most recent first)
        all_logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Take the most recent logs
        all_logs.into_iter().take(limit).collect()
    }

    fn parse_log_line(&self, line: &str, source: &str) -> Option<LogEntry> {
        // Handle ANSI color codes in log lines
        let clean_line = strip_ansi_escapes::strip_str(line);

        // Parse different log formats
        // Example formats:
        // 2025-12-10T02:15:28.585930Z  INFO nullblock_agents::handlers::health: ✅ Hecate agent healthy
        // [2m2025-12-10T02:15:28.585930Z[0m [32m INFO[0m [1;32mnullblock_agents[0m: message

        let level = if clean_line.contains("ERROR") || clean_line.contains("error") {
            LogLevel::Error
        } else if clean_line.contains("WARN") || clean_line.contains("warn") {
            LogLevel::Warning
        } else if clean_line.contains("DEBUG") || clean_line.contains("debug") {
            LogLevel::Debug
        } else if clean_line.contains("INFO") || clean_line.contains("info") {
            LogLevel::Info
        } else {
            LogLevel::Debug
        };

        let category = self.categorize_log(&clean_line);

        // Extract timestamp or use current time
        let timestamp = self
            .extract_timestamp(&clean_line)
            .unwrap_or_else(|| Utc::now().to_rfc3339());

        // Extract the actual message (remove timestamp and level prefixes)
        let message = self.extract_message(&clean_line);

        Some(LogEntry {
            timestamp,
            level,
            source: source.to_string(),
            message,
            category,
            metadata: HashMap::new(),
            sanitized: true,
        })
    }

    fn extract_timestamp(&self, line: &str) -> Option<String> {
        // Try to find ISO 8601 timestamp
        if let Some(start) = line.find("20") {
            if let Some(end) = line[start..].find("Z") {
                let timestamp_str = &line[start..start + end + 1];
                return Some(timestamp_str.to_string());
            }
        }
        None
    }

    fn extract_message(&self, line: &str) -> String {
        // Remove timestamp and log level prefixes
        let mut msg = line.to_string();

        // Remove timestamp
        if let Some(idx) = msg.find("Z") {
            msg = msg[idx + 1..].trim().to_string();
        }

        // Remove log level
        for level in &["ERROR", "WARN", "INFO", "DEBUG", "TRACE"] {
            if let Some(idx) = msg.find(level) {
                msg = msg[idx + level.len()..].trim().to_string();
                break;
            }
        }

        // Remove module path (e.g., "nullblock_agents::handlers::health:")
        if let Some(idx) = msg.find(": ") {
            msg = msg[idx + 2..].to_string();
        }

        msg
    }

    fn categorize_log(&self, line: &str) -> LogCategory {
        let lower = line.to_lowercase();

        if lower.contains("hecate") || lower.contains("chat") || lower.contains("agent") {
            LogCategory::AgentConversation
        } else if lower.contains("openrouter") || lower.contains("llm") || lower.contains("model") {
            LogCategory::LlmApiCall
        } else if lower.contains("erebus") || lower.contains("routing") || lower.contains("proxy") {
            LogCategory::ErebusRouting
        } else if lower.contains("task") || lower.contains("lifecycle") {
            LogCategory::TaskLifecycle
        } else if lower.contains("health") || lower.contains("heartbeat") {
            LogCategory::HealthCheck
        } else if lower.contains("error") || lower.contains("fail") || lower.contains("critical") {
            LogCategory::SystemError
        } else if lower.contains("database") || lower.contains("postgres") || lower.contains("sql")
        {
            LogCategory::DatabaseQuery
        } else {
            LogCategory::Unknown
        }
    }
}
