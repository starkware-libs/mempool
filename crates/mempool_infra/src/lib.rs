use async_trait::async_trait;
// use papyrus_config::dumping::SerializeConfig;
use std::fmt::Debug;

#[cfg(test)]
mod infra_test;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ComponentStartError {
    #[error("Error in the component configuration.")]
    ComponentConfigError,
    #[error("An internal component error.")]
    InternalComponentError,
}

/// Interface to start memepool components.
#[async_trait]
pub trait ComponentRunner {
    /// Start the component. Normally this function should never return.
    async fn start_component(&self) -> Result<(), ComponentStartError>;
}
