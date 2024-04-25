use starknet_api::transaction::TransactionHash;
use starknet_api::StarknetApiError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MempoolError {
    #[error("Duplicate transaction, of hash: {tx_hash}")]
    DuplicateTransaction { tx_hash: TransactionHash },
    #[error(transparent)]
    StarknetApiError(#[from] StarknetApiError),
}
