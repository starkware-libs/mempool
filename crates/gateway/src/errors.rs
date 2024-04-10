use blockifier::{
    blockifier::stateful_validator::StatefulValidatorError,
    transaction::errors::TransactionExecutionError,
};
use starknet_api::{
    block::BlockNumber,
    transaction::{Resource, ResourceBounds}, StarknetApiError,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error(transparent)]
    HTTPError(#[from] hyper::http::Error),
    #[error("Internal server error")]
    InternalServerError,
    #[error(transparent)]
    InvalidTransactionFormat(#[from] serde_json::Error),
    #[error("Error while starting the server")]
    ServerStartError(#[from] hyper::Error),
}

#[derive(Debug, Error)]
pub enum TransactionValidatorError {
    #[error("Block number {block_number:?} is out of range.")]
    BlockNumberOutOfRange { block_number: BlockNumber },
    #[error("Expected a positive amount of {resource:?}. Got {resource_bounds:?}.")]
    ZeroFee {
        resource: Resource,
        resource_bounds: ResourceBounds,
    },
    #[error("The resource bounds mapping is missing a resource {resource:?}.")]
    MissingResource { resource: Resource },
    #[error(transparent)]
    StarknetApiError(#[from] StarknetApiError),
    #[error(transparent)]
    StatefulValidatorError(#[from] StatefulValidatorError),
    #[error(transparent)]
    TransactionExecutionError(#[from] TransactionExecutionError),
}

pub type TransactionValidatorResult<T> = Result<T, TransactionValidatorError>;
