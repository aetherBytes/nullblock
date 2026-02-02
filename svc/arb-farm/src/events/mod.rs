mod types;
pub mod topics;
mod bus;

pub use types::*;
pub use topics::*;
pub use bus::*;

pub fn broadcast_event(tx: &tokio::sync::broadcast::Sender<ArbEvent>, event: ArbEvent) {
    if let Err(e) = tx.send(event) {
        tracing::warn!("Failed to broadcast event: {}", e);
    }
}
