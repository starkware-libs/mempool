use starknet_api::core::ContractAddress;
use starknet_api::transaction::TransactionHash;
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum MempoolError {
    #[error("Duplicate transaction, with hash: {tx_hash}")]
    DuplicateTransaction { tx_hash: TransactionHash },
    #[error("Undeployed account {:?}", sender_address)]
    UndeployedAccount { sender_address: ContractAddress },
    #[error("Transaction with hash: {tx_hash} not found")]
    TransactionNotFound { tx_hash: TransactionHash },
}
