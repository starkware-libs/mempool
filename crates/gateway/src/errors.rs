use starknet_api::transaction::TransactionVersion;

use thiserror::Error;

#[derive(Debug, Error)]
#[cfg_attr(test, derive(PartialEq))]
pub enum StarknetApiTransactionError {
    #[error("This transaction type is not supported by the mempool")]
    TransactionTypeNotSupported,
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
}

pub type TransactionValidatorResult<T> = Result<T, TransactionValidatorError>;
