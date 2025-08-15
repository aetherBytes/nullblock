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