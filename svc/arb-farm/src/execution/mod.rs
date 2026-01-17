pub mod approval_manager;
pub mod capital_manager;
pub mod curve_builder;
pub mod jito;
pub mod simulation;
pub mod executor;
pub mod risk;
pub mod priority_queue;
pub mod blockhash;
pub mod transaction_builder;
pub mod position_manager;
pub mod position_monitor;

pub use approval_manager::ApprovalManager;
pub use capital_manager::{CapitalManager, CapitalError, CapitalReservation, StrategyAllocation, StrategyUsage, GlobalCapitalUsage};
pub use curve_builder::{CurveTransactionBuilder, CurveBuildResult, CurveBuyParams, CurveSellParams, SimulatedTrade};
pub use jito::{JitoClient, BundleSubmission, BundleStatus};
pub use simulation::{SimulationResult, TransactionSimulator};
pub use executor::{ExecutorAgent, ExecutionResult};
pub use risk::{RiskManager, RiskCheck, RiskViolation};
pub use priority_queue::{EdgePriorityQueue, Priority, PrioritizedEdge, QueueStats};
pub use blockhash::{BlockhashCache, RecentBlockhash};
pub use transaction_builder::{TransactionBuilder, BuildResult, SwapParams, RouteInfo, ExitBuildResult};
pub use position_manager::{
    PositionManager, OpenPosition, ExitConfig, ExitSignal, ExitReason, ExitMode,
    BaseCurrency, PositionStatus, SOL_MINT, USDC_MINT, USDT_MINT,
};
pub use position_monitor::{PositionMonitor, MonitorConfig};
