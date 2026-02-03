mod bus;
pub mod topics;
mod types;

pub use bus::*;
pub use topics::*;
pub use types::*;

pub fn broadcast_event(tx: &tokio::sync::broadcast::Sender<ArbEvent>, event: ArbEvent) {
    if let Err(e) = tx.send(event) {
        tracing::warn!("Failed to broadcast event: {}", e);
    }
}
