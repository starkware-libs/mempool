use thiserror::Error;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error(transparent)]
    ConfigError(#[from] GatewayConfigError),
    #[error("Internal server error")]
    InternalServerError,
    #[error("Invalid transaction format")]
    InvalidTransactionFormat,
    #[error("Error while starting the server")]
    ServerStartError,
}

#[derive(Debug, Error)]
pub enum GatewayConfigError {
    #[error("Server address is not an bind IP address: {0}")]
    InvalidServerBindAddress(String),
}
