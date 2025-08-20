// Agent routing module for Erebus
// Handles communication between frontend and backend agent services

pub mod routes;
pub mod proxy;

pub use routes::*;
pub use proxy::*;