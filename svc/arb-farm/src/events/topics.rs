pub mod scanner {
    pub const ALL: &str = "arb.scanner.*";
    pub const SIGNAL_DETECTED: &str = "arb.scanner.signal.detected";
    pub const VENUE_ADDED: &str = "arb.scanner.venue.added";
    pub const VENUE_REMOVED: &str = "arb.scanner.venue.removed";
    pub const STARTED: &str = "arb.scanner.started";
    pub const STOPPED: &str = "arb.scanner.stopped";
}

pub mod edge {
    pub const ALL: &str = "arb.edge.*";
    pub const DETECTED: &str = "arb.edge.detected";
    pub const APPROVED: &str = "arb.edge.approved";
    pub const REJECTED: &str = "arb.edge.rejected";
    pub const EXECUTING: &str = "arb.edge.executing";
    pub const EXECUTED: &str = "arb.edge.executed";
    pub const FAILED: &str = "arb.edge.failed";
    pub const EXPIRED: &str = "arb.edge.expired";
}

pub mod strategy {
    pub const ALL: &str = "arb.strategy.*";
    pub const CREATED: &str = "arb.strategy.created";
    pub const UPDATED: &str = "arb.strategy.updated";
    pub const DELETED: &str = "arb.strategy.deleted";
    pub const TRIGGERED: &str = "arb.strategy.triggered";
    pub const ENABLED: &str = "arb.strategy.enabled";
    pub const DISABLED: &str = "arb.strategy.disabled";
}

pub mod research {
    pub const ALL: &str = "arb.research.*";
    pub const URL_INGESTED: &str = "arb.research.url.ingested";
    pub const STRATEGY_DISCOVERED: &str = "arb.research.strategy.discovered";
    pub const STRATEGY_APPROVED: &str = "arb.research.strategy.approved";
    pub const BACKTEST_COMPLETED: &str = "arb.research.backtest.completed";
}

pub mod kol {
    pub const ALL: &str = "arb.kol.*";
    pub const TRADE_DETECTED: &str = "arb.kol.trade.detected";
    pub const TRADE_COPIED: &str = "arb.kol.trade.copied";
    pub const TRUST_UPDATED: &str = "arb.kol.trust.updated";
    pub const ADDED: &str = "arb.kol.added";
    pub const REMOVED: &str = "arb.kol.removed";
}

pub mod threat {
    pub const ALL: &str = "arb.threat.*";
    pub const DETECTED: &str = "arb.threat.detected";
    pub const BLOCKED: &str = "arb.threat.blocked";
    pub const ALERT: &str = "arb.threat.alert";
    pub const WHITELISTED: &str = "arb.threat.whitelisted";
}

pub mod engram {
    pub const ALL: &str = "arb.engram.*";
    pub const CREATED: &str = "arb.engram.created";
    pub const PATTERN_MATCHED: &str = "arb.engram.pattern.matched";
    pub const AVOIDANCE_CREATED: &str = "arb.engram.avoidance.created";
    pub const STRATEGY_OPTIMIZED: &str = "arb.engram.strategy.optimized";
}

pub mod swarm {
    pub const ALL: &str = "arb.swarm.*";
    pub const AGENT_STARTED: &str = "arb.swarm.agent.started";
    pub const AGENT_STOPPED: &str = "arb.swarm.agent.stopped";
    pub const AGENT_FAILED: &str = "arb.swarm.agent.failed";
    pub const AGENT_RECOVERED: &str = "arb.swarm.agent.recovered";
    pub const PAUSED: &str = "arb.swarm.paused";
    pub const RESUMED: &str = "arb.swarm.resumed";
}

pub mod consensus {
    pub const ALL: &str = "arb.consensus.*";
    pub const REQUESTED: &str = "arb.consensus.requested";
    pub const COMPLETED: &str = "arb.consensus.completed";
    pub const FAILED: &str = "arb.consensus.failed";
}

pub mod trade {
    pub const ALL: &str = "arb.trade.*";
    pub const SUBMITTED: &str = "arb.trade.submitted";
    pub const CONFIRMED: &str = "arb.trade.confirmed";
    pub const FAILED: &str = "arb.trade.failed";
}

pub fn matches_pattern(topic: &str, pattern: &str) -> bool {
    if pattern.ends_with(".*") {
        let prefix = &pattern[..pattern.len() - 2];
        topic.starts_with(prefix)
    } else {
        topic == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_pattern() {
        assert!(matches_pattern("arb.edge.detected", edge::ALL));
        assert!(matches_pattern("arb.edge.executed", edge::ALL));
        assert!(!matches_pattern("arb.scanner.signal.detected", edge::ALL));
        assert!(matches_pattern("arb.edge.detected", edge::DETECTED));
        assert!(!matches_pattern("arb.edge.executed", edge::DETECTED));
    }
}
