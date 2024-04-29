use async_trait::async_trait;
use papyrus_config::dumping::SerializeConfig;
use std::any::Any;
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

pub trait ExtendConfig: SerializeConfig {
    fn as_any(&self) -> &dyn Any;
}

impl<T: SerializeConfig + Sync + Send + 'static> ExtendConfig for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Interface to start memepool components.
#[async_trait]
pub trait ComponentRunner {
    /// Start the component. Normally this function should never return.
    async fn start_component(
        &self,
        config: Option<Box<&(dyn ExtendConfig + Sync + Send)>>,
    ) -> Result<(), ComponentStartError>;
}

pub fn get_config<T: ExtendConfig + 'static>(
    config: Option<Box<&(dyn ExtendConfig + Sync + Send)>>,
) -> Option<&T> {
    if let Some(config) = config {
        config.as_any().downcast_ref::<T>()
    } else {
        None
    }
}
