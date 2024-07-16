use axum::http::StatusCode;
use blockifier::state::errors::StateError;
use serde_json::{Error as SerdeError, Value};
use starknet_api::block::{GasPrice};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RPCStateReaderError {
    #[error("Block not found for request {0}")]
    BlockNotFound(Value),
    #[error("Class hash not found for request {0}")]
    ClassHashNotFound(Value),
    #[error("Failed to parse gas price {:?}", 0)]
    GasPriceParsingFailure(GasPrice),
    #[error("Contract address not found for request {0}")]
    ContractAddressNotFound(Value),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("RPC error: {0}")]
    RPCError(StatusCode),
    #[error("Unexpected error code: {0}")]
    UnexpectedErrorCode(u16),
}

pub type RPCStateReaderResult<T> = Result<T, RPCStateReaderError>;

impl From<RPCStateReaderError> for StateError {
    fn from(err: RPCStateReaderError) -> Self {
        match err {
            RPCStateReaderError::ClassHashNotFound(request) => {
                match serde_json::from_value(request["params"]["class_hash"].clone()) {
                    Ok(class_hash) => StateError::UndeclaredClassHash(class_hash),
                    Err(e) => serde_err_to_state_err(e),
                }
            }
            _ => StateError::StateReadError(err.to_string()),
        }
    }
}

// Converts a serde error to the error type of the state reader.
pub fn serde_err_to_state_err(err: SerdeError) -> StateError {
    StateError::StateReadError(format!("Failed to parse rpc result {:?}", err.to_string()))
}
