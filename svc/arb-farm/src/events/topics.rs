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
    pub const TRADE_COPY_FAILED: &str = "arb.kol.trade.copy_failed";
    pub const TRUST_UPDATED: &str = "arb.kol.trust.updated";
    pub const ADDED: &str = "arb.kol.added";
    pub const REMOVED: &str = "arb.kol.removed";
    pub const PROMOTED: &str = "arb.kol.promoted";
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

pub mod approval {
    pub const ALL: &str = "arb.approval.*";
    pub const CREATED: &str = "arb.approval.created";
    pub const APPROVED: &str = "arb.approval.approved";
    pub const REJECTED: &str = "arb.approval.rejected";
    pub const EXPIRED: &str = "arb.approval.expired";
    pub const AUTO_APPROVED: &str = "arb.approval.auto_approved";
    pub const CONFIG_UPDATED: &str = "arb.approval.config.updated";
    pub const EXECUTION_ENABLED: &str = "arb.approval.execution.enabled";
    pub const EXECUTION_DISABLED: &str = "arb.approval.execution.disabled";
    pub const HECATE_NOTIFIED: &str = "arb.approval.hecate.notified";
    pub const HECATE_RECOMMENDED: &str = "arb.approval.hecate.recommended";
}

pub mod trade {
    pub const ALL: &str = "arb.trade.*";
    pub const SUBMITTED: &str = "arb.trade.submitted";
    pub const CONFIRMED: &str = "arb.trade.confirmed";
    pub const FAILED: &str = "arb.trade.failed";
}

pub mod position {
    pub const ALL: &str = "arb.position.*";
    pub const OPENED: &str = "arb.position.opened";
    pub const UPDATED: &str = "arb.position.updated";
    pub const CLOSED: &str = "arb.position.closed";
    pub const EXIT_PENDING: &str = "arb.position.exit_pending";
    pub const EXIT_FAILED: &str = "arb.position.exit_failed";
    pub const STOP_LOSS_TRIGGERED: &str = "arb.position.sl_triggered";
    pub const TAKE_PROFIT_TRIGGERED: &str = "arb.position.tp_triggered";
    pub const TRAILING_STOP_TRIGGERED: &str = "arb.position.trailing_triggered";
    pub const MOMENTUM_EXIT: &str = "arb.position.momentum_exit";
    pub const EMERGENCY_EXIT: &str = "arb.position.emergency_exit";
    pub const BULK_CLEARED: &str = "arb.position.bulk_cleared";
}

pub mod curve {
    pub const ALL: &str = "arb.curve.*";
    pub const PROGRESS_MILESTONE: &str = "arb.curve.progress.milestone";
    pub const GRADUATION_IMMINENT: &str = "arb.curve.graduation_imminent";
    pub const GRADUATED: &str = "arb.curve.graduated";
}

pub mod helius {
    pub const ALL: &str = "arb.helius.*";

    pub mod laserstream {
        pub const ALL: &str = "arb.helius.laserstream.*";
        pub const CONNECTED: &str = "arb.helius.laserstream.connected";
        pub const DISCONNECTED: &str = "arb.helius.laserstream.disconnected";
        pub const ACCOUNT: &str = "arb.helius.laserstream.account";
        pub const TRANSACTION: &str = "arb.helius.laserstream.transaction";
        pub const SLOT: &str = "arb.helius.laserstream.slot";
        pub const BLOCK: &str = "arb.helius.laserstream.block";
    }

    pub mod priority_fee {
        pub const ALL: &str = "arb.helius.priority_fee.*";
        pub const UPDATED: &str = "arb.helius.priority_fee.updated";
    }

    pub mod sender {
        pub const ALL: &str = "arb.helius.sender.*";
        pub const TX_SENT: &str = "arb.helius.sender.tx_sent";
        pub const TX_CONFIRMED: &str = "arb.helius.sender.tx_confirmed";
        pub const TX_FAILED: &str = "arb.helius.sender.tx_failed";
    }

    pub mod das {
        pub const ALL: &str = "arb.helius.das.*";
        pub const METADATA_FETCHED: &str = "arb.helius.das.metadata_fetched";
        pub const CREATOR_FLAGGED: &str = "arb.helius.das.creator_flagged";
    }

    pub mod webhook {
        pub const ALL: &str = "arb.helius.webhook.*";
        pub const RECEIVED: &str = "arb.helius.webhook.received";
        pub const KOL_TRADE: &str = "arb.helius.webhook.kol_trade";
    }
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
