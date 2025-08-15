// Worker factory for delegating complex MCP operations to nullblock.mcp service
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct McpWorker {
    pub worker_id: String,
    pub operation: String,
    pub status: WorkerStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub params: Value,
    pub result: Option<Value>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WorkerStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Delegated,
}

pub struct McpWorkerFactory {
    workers: std::sync::Arc<std::sync::Mutex<HashMap<String, McpWorker>>>,
    nullblock_mcp_url: String,
}

impl McpWorkerFactory {
    pub fn new() -> Self {
        Self {
            workers: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            nullblock_mcp_url: std::env::var("NULLBLOCK_MCP_URL").unwrap_or_else(|_| "http://localhost:8000".to_string()),
        }
    }

    /// Create a new worker for the given operation
    pub fn create_worker(&self, operation: &str, params: Value) -> McpWorker {
        let worker_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        
        let worker = McpWorker {
            worker_id: worker_id.clone(),
            operation: operation.to_string(),
            status: WorkerStatus::Pending,
            created_at: now,
            updated_at: now,
            params,
            result: None,
            error: None,
        };

        // Store worker
        {
            let mut workers = self.workers.lock().unwrap();
            workers.insert(worker_id.clone(), worker.clone());
        }

        println!("🏭 Created MCP worker {} for operation '{}'", worker_id, operation);
        worker
    }

    /// Submit worker to nullblock.mcp service
    pub async fn submit_to_nullblock_mcp(&self, worker_id: &str) -> Result<Value, String> {
        let worker = {
            let workers = self.workers.lock().unwrap();
            workers.get(worker_id).cloned()
        };

        let mut worker = match worker {
            Some(w) => w,
            None => return Err(format!("Worker {} not found", worker_id)),
        };

        // Update status to delegated
        worker.status = WorkerStatus::Delegated;
        worker.updated_at = chrono::Utc::now();
        
        {
            let mut workers = self.workers.lock().unwrap();
            workers.insert(worker_id.to_string(), worker.clone());
        }

        println!("🔄 Submitting worker {} to nullblock.mcp at {}", worker_id, self.nullblock_mcp_url);

        // Create payload for nullblock.mcp
        let payload = serde_json::json!({
            "worker_id": worker_id,
            "operation": worker.operation,
            "params": worker.params,
            "erebus_callback": format!("http://localhost:3000/mcp/worker/{}/callback", worker_id)
        });

        // TODO: Replace with actual HTTP client call
        let response = self.mock_nullblock_mcp_call(&worker.operation, &payload).await;

        match response {
            Ok(result) => {
                // Update worker with result
                worker.status = WorkerStatus::Completed;
                worker.result = Some(result.clone());
                worker.updated_at = chrono::Utc::now();
                
                {
                    let mut workers = self.workers.lock().unwrap();
                    workers.insert(worker_id.to_string(), worker);
                }

                Ok(result)
            }
            Err(error) => {
                // Update worker with error
                worker.status = WorkerStatus::Failed;
                worker.error = Some(error.clone());
                worker.updated_at = chrono::Utc::now();
                
                {
                    let mut workers = self.workers.lock().unwrap();
                    workers.insert(worker_id.to_string(), worker);
                }

                Err(error)
            }
        }
    }

    /// Mock nullblock.mcp service call (replace with real HTTP client)
    async fn mock_nullblock_mcp_call(&self, operation: &str, _payload: &Value) -> Result<Value, String> {
        println!("📡 [MOCK] Calling nullblock.mcp for operation: {}", operation);
        
        // Simulate processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        match operation {
            "social_trading" => Ok(serde_json::json!({
                "analysis": {
                    "sentiment_score": 0.75,
                    "confidence": 0.88,
                    "signals": [
                        {"source": "twitter", "mentions": 1247, "sentiment": "bullish"},
                        {"source": "telegram", "mentions": 892, "sentiment": "neutral"}
                    ],
                    "recommendation": "BUY",
                    "risk_level": "MODERATE"
                },
                "processing_time_ms": 1250,
                "data_sources": ["twitter", "telegram", "dextools"],
                "worker_id": "mcp-worker-123"
            })),
            "arbitrage" => Ok(serde_json::json!({
                "opportunities": [
                    {
                        "pair": "SOL/USDC",
                        "buy_exchange": "Jupiter",
                        "sell_exchange": "Raydium", 
                        "profit_percentage": 2.3,
                        "mev_protection": true,
                        "execution_time_estimate": "15s"
                    }
                ],
                "total_opportunities": 1,
                "estimated_profit_usd": 147.50,
                "worker_id": "mcp-worker-456"
            })),
            "resource_trading_agents" => Ok(serde_json::json!({
                "active_agents": [
                    {"type": "social_trading", "status": "running", "success_rate": 0.73},
                    {"type": "arbitrage", "status": "running", "success_rate": 0.91},
                    {"type": "price_monitoring", "status": "idle", "success_rate": 0.95}
                ],
                "total_trades_today": 42,
                "total_profit_today": 1247.85
            })),
            "prompt_trading_strategy" => Ok(serde_json::json!({
                "messages": [
                    {
                        "role": "assistant",
                        "content": {
                            "type": "text",
                            "text": "Based on current market conditions and your risk profile, I recommend a diversified approach focusing on established tokens with strong fundamentals. Consider dollar-cost averaging into SOL and ETH while maintaining 20% cash reserves for opportunities..."
                        }
                    }
                ]
            })),
            _ => Err(format!("Unknown operation: {}", operation))
        }
    }

    /// Get worker status
    pub fn get_worker_status(&self, worker_id: &str) -> Option<McpWorker> {
        let workers = self.workers.lock().unwrap();
        workers.get(worker_id).cloned()
    }

    /// Get all workers
    pub fn get_all_workers(&self) -> Vec<McpWorker> {
        let workers = self.workers.lock().unwrap();
        workers.values().cloned().collect()
    }

    /// Clean up completed workers older than 1 hour
    pub fn cleanup_old_workers(&self) {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(1);
        let mut workers = self.workers.lock().unwrap();
        
        let initial_count = workers.len();
        workers.retain(|_, worker| {
            worker.updated_at > cutoff || 
            (worker.status == WorkerStatus::Pending || worker.status == WorkerStatus::Running)
        });
        
        let removed_count = initial_count - workers.len();
        if removed_count > 0 {
            println!("🧹 Cleaned up {} old MCP workers", removed_count);
        }
    }

    /// Get worker statistics
    pub fn get_worker_stats(&self) -> Value {
        let workers = self.workers.lock().unwrap();
        let total = workers.len();
        let mut status_counts = HashMap::new();

        for worker in workers.values() {
            *status_counts.entry(format!("{:?}", worker.status)).or_insert(0) += 1;
        }

        serde_json::json!({
            "total_workers": total,
            "status_breakdown": status_counts,
            "nullblock_mcp_url": self.nullblock_mcp_url
        })
    }
}