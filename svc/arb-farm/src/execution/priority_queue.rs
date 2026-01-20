use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::events::AtomicityLevel;
use crate::models::{Edge, EdgeStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Critical = 4,
    High = 3,
    Medium = 2,
    Low = 1,
}

impl From<i32> for Priority {
    fn from(value: i32) -> Self {
        match value {
            4 => Priority::Critical,
            3 => Priority::High,
            2 => Priority::Medium,
            _ => Priority::Low,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrioritizedEdge {
    pub edge: Edge,
    pub priority: Priority,
    pub deadline: chrono::DateTime<chrono::Utc>,
    pub enqueued_at: chrono::DateTime<chrono::Utc>,
    pub retry_count: u32,
}

impl PrioritizedEdge {
    pub fn new(edge: Edge) -> Self {
        let priority = Self::calculate_priority(&edge);
        let deadline = edge.expires_at.unwrap_or(chrono::Utc::now() + chrono::Duration::minutes(5));

        Self {
            edge,
            priority,
            deadline,
            enqueued_at: chrono::Utc::now(),
            retry_count: 0,
        }
    }

    pub fn with_priority(edge: Edge, priority: Priority) -> Self {
        let deadline = edge.expires_at.unwrap_or(chrono::Utc::now() + chrono::Duration::minutes(5));

        Self {
            edge,
            priority,
            deadline,
            enqueued_at: chrono::Utc::now(),
            retry_count: 0,
        }
    }

    fn calculate_priority(edge: &Edge) -> Priority {
        match edge.atomicity {
            AtomicityLevel::FullyAtomic => {
                if edge.simulated_profit_guaranteed {
                    Priority::Critical
                } else {
                    Priority::High
                }
            }
            AtomicityLevel::PartiallyAtomic => Priority::Medium,
            AtomicityLevel::NonAtomic => {
                let profit = edge.estimated_profit_lamports.unwrap_or(0);
                if profit > 1_000_000_000 {
                    Priority::High
                } else if profit > 100_000_000 {
                    Priority::Medium
                } else {
                    Priority::Low
                }
            }
        }
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.deadline
    }

    pub fn time_remaining_ms(&self) -> i64 {
        (self.deadline - chrono::Utc::now()).num_milliseconds()
    }

    pub fn urgency_score(&self) -> i64 {
        let time_remaining = self.time_remaining_ms();
        let priority_bonus = (self.priority as i64) * 10000;
        let profit_bonus = self.edge.estimated_profit_lamports.unwrap_or(0) / 1000;

        priority_bonus + profit_bonus - (time_remaining / 100).max(0)
    }
}

impl PartialEq for PrioritizedEdge {
    fn eq(&self, other: &Self) -> bool {
        self.edge.id == other.edge.id
    }
}

impl Eq for PrioritizedEdge {}

impl PartialOrd for PrioritizedEdge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedEdge {
    fn cmp(&self, other: &Self) -> Ordering {
        self.urgency_score().cmp(&other.urgency_score())
    }
}

pub struct EdgePriorityQueue {
    queue: Arc<RwLock<BinaryHeap<PrioritizedEdge>>>,
    max_size: usize,
    stats: Arc<RwLock<QueueStats>>,
}

#[derive(Debug, Clone, Default)]
pub struct QueueStats {
    pub total_enqueued: u64,
    pub total_dequeued: u64,
    pub total_expired: u64,
    pub total_retried: u64,
    pub current_size: usize,
    pub by_priority: PriorityBreakdown,
}

#[derive(Debug, Clone, Default)]
pub struct PriorityBreakdown {
    pub critical: u64,
    pub high: u64,
    pub medium: u64,
    pub low: u64,
}

impl EdgePriorityQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Arc::new(RwLock::new(BinaryHeap::new())),
            max_size,
            stats: Arc::new(RwLock::new(QueueStats::default())),
        }
    }

    pub async fn enqueue(&self, edge: Edge) -> bool {
        let prioritized = PrioritizedEdge::new(edge);
        self.enqueue_prioritized(prioritized).await
    }

    pub async fn enqueue_with_priority(&self, edge: Edge, priority: Priority) -> bool {
        let prioritized = PrioritizedEdge::with_priority(edge, priority);
        self.enqueue_prioritized(prioritized).await
    }

    async fn enqueue_prioritized(&self, prioritized: PrioritizedEdge) -> bool {
        if prioritized.is_expired() {
            let mut stats = self.stats.write().await;
            stats.total_expired += 1;
            return false;
        }

        let mut queue = self.queue.write().await;

        if queue.len() >= self.max_size {
            if let Some(lowest) = queue.peek() {
                if prioritized.urgency_score() <= lowest.urgency_score() {
                    return false;
                }
            }
            queue.pop();
        }

        let priority = prioritized.priority;
        queue.push(prioritized);

        let mut stats = self.stats.write().await;
        stats.total_enqueued += 1;
        stats.current_size = queue.len();

        match priority {
            Priority::Critical => stats.by_priority.critical += 1,
            Priority::High => stats.by_priority.high += 1,
            Priority::Medium => stats.by_priority.medium += 1,
            Priority::Low => stats.by_priority.low += 1,
        }

        true
    }

    pub async fn dequeue(&self) -> Option<PrioritizedEdge> {
        let mut queue = self.queue.write().await;

        while let Some(edge) = queue.pop() {
            if edge.is_expired() {
                let mut stats = self.stats.write().await;
                stats.total_expired += 1;
                stats.current_size = queue.len();
                continue;
            }

            let mut stats = self.stats.write().await;
            stats.total_dequeued += 1;
            stats.current_size = queue.len();
            return Some(edge);
        }

        None
    }

    pub async fn dequeue_batch(&self, count: usize) -> Vec<PrioritizedEdge> {
        let mut batch = Vec::with_capacity(count);

        for _ in 0..count {
            if let Some(edge) = self.dequeue().await {
                batch.push(edge);
            } else {
                break;
            }
        }

        batch
    }

    pub async fn peek(&self) -> Option<PrioritizedEdge> {
        let queue = self.queue.read().await;
        queue.peek().cloned()
    }

    pub async fn remove(&self, edge_id: Uuid) -> bool {
        let mut queue = self.queue.write().await;
        let original_len = queue.len();

        let items: Vec<PrioritizedEdge> = queue.drain().collect();
        let filtered: Vec<PrioritizedEdge> = items
            .into_iter()
            .filter(|e| e.edge.id != edge_id)
            .collect();

        for item in filtered {
            queue.push(item);
        }

        let mut stats = self.stats.write().await;
        stats.current_size = queue.len();

        queue.len() < original_len
    }

    pub async fn requeue_with_retry(&self, mut edge: PrioritizedEdge) -> bool {
        edge.retry_count += 1;

        if edge.retry_count > 3 {
            return false;
        }

        edge.deadline = chrono::Utc::now() + chrono::Duration::seconds(5);

        let result = self.enqueue_prioritized(edge).await;

        if result {
            let mut stats = self.stats.write().await;
            stats.total_retried += 1;
        }

        result
    }

    pub async fn cleanup_expired(&self) -> u64 {
        let mut queue = self.queue.write().await;
        let original_len = queue.len();

        let items: Vec<PrioritizedEdge> = queue.drain().collect();
        let valid: Vec<PrioritizedEdge> = items
            .into_iter()
            .filter(|e| !e.is_expired())
            .collect();

        let expired_count = original_len - valid.len();

        for item in valid {
            queue.push(item);
        }

        let mut stats = self.stats.write().await;
        stats.total_expired += expired_count as u64;
        stats.current_size = queue.len();

        expired_count as u64
    }

    pub async fn len(&self) -> usize {
        self.queue.read().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.queue.read().await.is_empty()
    }

    pub async fn get_stats(&self) -> QueueStats {
        self.stats.read().await.clone()
    }

    pub async fn get_by_priority(&self, priority: Priority) -> Vec<PrioritizedEdge> {
        let queue = self.queue.read().await;
        queue
            .iter()
            .filter(|e| e.priority == priority)
            .cloned()
            .collect()
    }

    pub async fn get_critical_edges(&self) -> Vec<PrioritizedEdge> {
        self.get_by_priority(Priority::Critical).await
    }

    pub async fn get_atomic_edges(&self) -> Vec<PrioritizedEdge> {
        let queue = self.queue.read().await;
        queue
            .iter()
            .filter(|e| e.edge.atomicity == AtomicityLevel::FullyAtomic)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_edge(profit: i64, atomicity: AtomicityLevel) -> Edge {
        Edge {
            id: Uuid::new_v4(),
            strategy_id: None,
            edge_type: "test".to_string(),
            execution_mode: "autonomous".to_string(),
            atomicity,
            simulated_profit_guaranteed: atomicity == AtomicityLevel::FullyAtomic,
            estimated_profit_lamports: Some(profit),
            risk_score: Some(10),
            route_data: serde_json::json!({}),
            signal_data: None,
            status: EdgeStatus::Detected,
            token_mint: None,
            created_at: chrono::Utc::now(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::minutes(5)),
        }
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let queue = EdgePriorityQueue::new(100);

        let low_edge = make_test_edge(100_000, AtomicityLevel::NonAtomic);
        let high_edge = make_test_edge(1_000_000_000, AtomicityLevel::FullyAtomic);

        queue.enqueue(low_edge).await;
        queue.enqueue(high_edge.clone()).await;

        let dequeued = queue.dequeue().await.unwrap();
        assert_eq!(dequeued.edge.atomicity, AtomicityLevel::FullyAtomic);
    }

    #[tokio::test]
    async fn test_max_size_eviction() {
        let queue = EdgePriorityQueue::new(2);

        let edge1 = make_test_edge(100, AtomicityLevel::NonAtomic);
        let edge2 = make_test_edge(200, AtomicityLevel::NonAtomic);
        let edge3 = make_test_edge(10_000_000_000, AtomicityLevel::FullyAtomic);

        queue.enqueue(edge1).await;
        queue.enqueue(edge2).await;
        queue.enqueue(edge3).await;

        assert_eq!(queue.len().await, 2);
    }
}
