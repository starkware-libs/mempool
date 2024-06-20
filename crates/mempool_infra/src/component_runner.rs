use std::sync::Arc;

use async_trait::async_trait;
use papyrus_config::dumping::SerializeConfig;

use crate::component_client::ComponentClient;

#[cfg(test)]
#[path = "component_runner_test.rs"]
mod component_runner_test;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ComponentStartError {
    #[error("Error in the component configuration.")]
    ComponentConfigError,
    #[error("An internal component error.")]
    InternalComponentError,
}

/// Interface to create memepool components.
pub trait ComponentCreator<Config, Request, Response>
where
    Config: SerializeConfig,
    Request: Send + Sync,
    Response: Send + Sync,
{
    fn create(config: Config, comm_client: Option<Arc<ComponentClient<Request, Response>>>)
    -> Self;
}

/// Interface to start memepool components.
#[async_trait]
pub trait ComponentRunner {
    /// Start the component. Normally this function should never return.
    async fn start(&mut self) -> Result<(), ComponentStartError>;
}
