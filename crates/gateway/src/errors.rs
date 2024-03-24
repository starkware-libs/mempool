use starknet_api::transaction::TransactionVersion;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error(transparent)]
    ConfigError(#[from] GatewayConfigError),
    #[error(transparent)]
    HTTPError(#[from] hyper::http::Error),
    #[error("Internal server error")]
    InternalServerError,
    #[error("Error while starting the server")]
    ServerStartError(#[from] hyper::Error),
}

#[derive(Debug, Error)]
pub enum GatewayConfigError {
    #[error("Server address is not an bind IP address: {0}")]
    InvalidServerBindAddress(String),
}

#[derive(Debug, Error)]
#[cfg_attr(test, derive(PartialEq))]
pub enum StarknetApiTransactionError {
    #[error("This transaction type is not supported by the mempool")]
    TransactionTypeNotSupported,
    // This error should never be returned to the user.
    #[error("An unsupported action was called for a transaction from the starknet API.")]
    TransactionDoesNotSupportAcction,
}

pub type StarknetApiTransactionResult<T> = Result<T, StarknetApiTransactionError>;

#[derive(Debug, Error)]
#[cfg_attr(test, derive(PartialEq))]
pub enum TransactionValidatorError {
    #[error(transparent)]
    StarknetApiTransactionError(#[from] StarknetApiTransactionError),
    #[error("Transactions of version {0:?} are not valid. {1}")]
    InvalidTransactionVersion(TransactionVersion, String),
    #[error("Blocked transaction version {0:?}. {1}")]
    BlockedTransactionVersion(TransactionVersion, String),
    #[error("Transaction must commit to pay a positive amount on fee.")]
    ZeroFee,
    #[error(
        "Calldata length exceeded maximum: length {calldata_length}
        (allowed length: {max_calldata_length})."
    )]
    CalldataTooLong {
        calldata_length: usize,
        max_calldata_length: usize,
    },
    #[error(
        "Signature length exceeded maximum: length {signature_length}
        (allowed length: {max_signature_length})."
    )]
    SignatureTooLong {
        signature_length: usize,
        max_signature_length: usize,
    },
}

pub type TransactionValidatorResult<T> = Result<T, TransactionValidatorError>;
