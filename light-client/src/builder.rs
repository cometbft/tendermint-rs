//! DSL for building light clients and supervisor

mod light_client;
pub use light_client::LightClientBuilder;

mod supervisor;
pub use supervisor::SupervisorBuilder;

pub mod error;
