use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout_duration: Duration,
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout_duration: Duration::from_secs(30),
            half_open_max_calls: 3,
        }
    }
}

#[derive(Debug)]
struct CircuitBreakerState {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    half_open_calls: u32,
    config: CircuitBreakerConfig,
}

impl CircuitBreakerState {
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            half_open_calls: 0,
            config,
        }
    }

    fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.config.timeout_duration {
                        self.state = CircuitState::HalfOpen;
                        self.half_open_calls = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    self.state = CircuitState::Closed;
                    true
                }
            }
            CircuitState::HalfOpen => {
                if self.half_open_calls < self.config.half_open_max_calls {
                    self.half_open_calls += 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                    self.half_open_calls = 0;
                }
            }
            CircuitState::Open => {}
        }
    }

    fn record_failure(&mut self) {
        self.last_failure_time = Some(Instant::now());

        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.config.failure_threshold {
                    self.state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.success_count = 0;
                self.half_open_calls = 0;
            }
            CircuitState::Open => {}
        }
    }
}

pub struct CircuitBreaker {
    name: String,
    state: Arc<RwLock<CircuitBreakerState>>,
}

impl CircuitBreaker {
    pub fn new(name: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        Self {
            name: name.into(),
            state: Arc::new(RwLock::new(CircuitBreakerState::new(config))),
        }
    }

    pub async fn can_execute(&self) -> bool {
        self.state.write().await.can_execute()
    }

    pub async fn record_success(&self) {
        self.state.write().await.record_success();
    }

    pub async fn record_failure(&self) {
        self.state.write().await.record_failure();
    }

    pub async fn state(&self) -> CircuitState {
        self.state.read().await.state.clone()
    }

    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        state.state = CircuitState::Closed;
        state.failure_count = 0;
        state.success_count = 0;
        state.half_open_calls = 0;
        state.last_failure_time = None;
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn execute<F, Fut, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        if !self.can_execute().await {
            return Err(CircuitBreakerError::Open);
        }

        match f().await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(e) => {
                self.record_failure().await;
                Err(CircuitBreakerError::Inner(e))
            }
        }
    }
}

impl Clone for CircuitBreaker {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            state: Arc::clone(&self.state),
        }
    }
}

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    Open,
    Inner(E),
}

impl<E: std::fmt::Display> std::fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::Open => write!(f, "Circuit breaker is open"),
            CircuitBreakerError::Inner(e) => write!(f, "{}", e),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for CircuitBreakerError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CircuitBreakerError::Open => None,
            CircuitBreakerError::Inner(e) => Some(e),
        }
    }
}

pub struct CircuitBreakerRegistry {
    breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    default_config: CircuitBreakerConfig,
}

impl CircuitBreakerRegistry {
    pub fn new(default_config: CircuitBreakerConfig) -> Self {
        Self {
            breakers: Arc::new(RwLock::new(HashMap::new())),
            default_config,
        }
    }

    pub async fn get_or_create(&self, name: &str) -> CircuitBreaker {
        let mut breakers = self.breakers.write().await;
        if let Some(breaker) = breakers.get(name) {
            breaker.clone()
        } else {
            let breaker = CircuitBreaker::new(name, self.default_config.clone());
            breakers.insert(name.to_string(), breaker.clone());
            breaker
        }
    }

    pub async fn get(&self, name: &str) -> Option<CircuitBreaker> {
        self.breakers.read().await.get(name).cloned()
    }

    pub async fn reset_all(&self) {
        let breakers = self.breakers.read().await;
        for breaker in breakers.values() {
            breaker.reset().await;
        }
    }

    pub async fn get_all_states(&self) -> HashMap<String, CircuitState> {
        let breakers = self.breakers.read().await;
        let mut states = HashMap::new();
        for (name, breaker) in breakers.iter() {
            states.insert(name.clone(), breaker.state().await);
        }
        states
    }
}

impl Clone for CircuitBreakerRegistry {
    fn clone(&self) -> Self {
        Self {
            breakers: Arc::clone(&self.breakers),
            default_config: self.default_config.clone(),
        }
    }
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_closed() {
        let breaker = CircuitBreaker::new("test", CircuitBreakerConfig::default());
        assert!(breaker.can_execute().await);
        assert_eq!(breaker.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let breaker = CircuitBreaker::new("test", config);

        for _ in 0..3 {
            breaker.record_failure().await;
        }

        assert_eq!(breaker.state().await, CircuitState::Open);
        assert!(!breaker.can_execute().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let breaker = CircuitBreaker::new("test", config);

        for _ in 0..3 {
            breaker.record_failure().await;
        }

        assert_eq!(breaker.state().await, CircuitState::Open);

        breaker.reset().await;
        assert_eq!(breaker.state().await, CircuitState::Closed);
        assert!(breaker.can_execute().await);
    }
}
