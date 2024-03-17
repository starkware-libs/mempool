use starknet_api::transaction::TransactionVersion;
use thiserror::Error;

#[derive(Debug, Error)]
#[cfg_attr(test, derive(PartialEq))]
pub enum GatewayError {
    #[error("Internal server error")]
    InternalServerError,
    #[error("Error while starting the server")]
    ServerError,
    #[error("Invalid transaction: {0}")]
    TransactionValidatorError(#[from] TransactionValidatorError),
}

#[derive(Debug, Error)]
#[cfg_attr(test, derive(PartialEq))]
pub enum TransactionValidatorError {
    #[error("Invalid transaction type")]
    InvalidTransactionType,
    #[error("Transactions of version {0:?} are not valid. {1}")]
    InvalidTransactionVersion(TransactionVersion, String),
    #[error("Blocked transaction version {0:?}. {1}")]
    BlockedTransactionVersion(TransactionVersion, String),
}
