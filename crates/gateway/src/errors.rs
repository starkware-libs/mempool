use thiserror::Error;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("Internal server error")]
    InternalServerError,
    #[error("Error while starting the server")]
    ServerError,
}
