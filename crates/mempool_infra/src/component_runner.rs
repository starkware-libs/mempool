use async_trait::async_trait;
use papyrus_config::dumping::SerializeConfig;

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

/// Interface to create components.
pub trait ComponentCreator<T: SerializeConfig> {
    fn create(config: T) -> Self;
}

/// Interface to start components.
#[async_trait]
pub trait ComponentRunner {
    /// Start the component. Normally this function should never return.
    async fn start(&mut self) -> Result<(), ComponentStartError>;
}
