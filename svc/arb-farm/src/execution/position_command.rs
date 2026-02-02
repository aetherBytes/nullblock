use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::position_manager::{ExitSignal, ExitUrgency};

#[derive(Debug, Clone)]
pub enum PositionCommand {
    Exit(ExitCommand),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandSource {
    Monitor,
    PriorityRetry,
    PendingRetry,
    ManualTrigger,
    CopyTradeEmergency,
}

#[derive(Debug, Clone)]
pub struct ExitCommand {
    pub signal: ExitSignal,
    pub source: CommandSource,
    pub queued_at: DateTime<Utc>,
}

impl ExitCommand {
    pub fn new(signal: ExitSignal, source: CommandSource) -> Self {
        Self {
            signal,
            source,
            queued_at: Utc::now(),
        }
    }
}

impl PositionCommand {
    pub fn urgency(&self) -> ExitUrgency {
        match self {
            PositionCommand::Exit(cmd) => cmd.signal.urgency,
        }
    }

    pub fn position_id(&self) -> Uuid {
        match self {
            PositionCommand::Exit(cmd) => cmd.signal.position_id,
        }
    }

    pub fn urgency_sort_key(&self) -> u8 {
        match self.urgency() {
            ExitUrgency::Critical => 0,
            ExitUrgency::High => 1,
            ExitUrgency::Medium => 2,
            ExitUrgency::Low => 3,
        }
    }
}

impl std::fmt::Display for CommandSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandSource::Monitor => write!(f, "Monitor"),
            CommandSource::PriorityRetry => write!(f, "PriorityRetry"),
            CommandSource::PendingRetry => write!(f, "PendingRetry"),
            CommandSource::ManualTrigger => write!(f, "ManualTrigger"),
            CommandSource::CopyTradeEmergency => write!(f, "CopyTradeEmergency"),
        }
    }
}
