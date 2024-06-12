use starknet_api::transaction::TransactionHash;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MempoolError {
    #[error("Duplicate transaction, of hash: {tx_hash}")]
    DuplicateTransaction { tx_hash: TransactionHash },
    #[error("Transaction of hash: {tx_hash} is not found")]
    TransactionNotFound { tx_hash: TransactionHash },
    #[error("Requested offset {requested} is larger than the maximum allowed {maximum}")]
    OffsetTooLarge { requested: usize, maximum: usize },
    #[error("Requested offset {requested} is smaller than the minimum allowed {minimum}")]
    OffsetTooSmall { requested: usize, minimum: usize },
}
