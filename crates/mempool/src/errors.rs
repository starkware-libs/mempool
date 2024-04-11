use starknet_api::transaction::TransactionHash;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MempoolError {
    #[error("This transaction has already been processed: tx_hash={tx_hash}")]
    DuplicateTransaction { tx_hash: TransactionHash },
}
