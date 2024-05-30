use std::cmp::Ordering;
use std::collections::HashMap;

use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::ThinTransaction;

#[derive(Clone, Debug, Default)]
pub struct StagingTransaction {
    pub tx_hash: TransactionHash,
    pub address: ContractAddress,
    pub nonce: Nonce,
}

impl PartialEq for StagingTransaction {
    fn eq(&self, other: &StagingTransaction) -> bool {
        self.address == other.address && self.nonce == other.nonce
    }
}

impl Eq for StagingTransaction {}

impl Ord for StagingTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address.cmp(&other.address).then_with(|| self.nonce.cmp(&other.nonce))
    }
}

impl PartialOrd for StagingTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Default, derive_more::Deref, derive_more::DerefMut)]
pub struct StagingQueue(HashMap<TransactionHash, StagingTransaction>);
impl StagingQueue {
    pub fn insert(&mut self, tx: StagingTransaction) -> Result<(), MempoolError> {
        if self.0.contains_key(&tx.tx_hash) {
            return Err(MempoolError::DuplicateTransaction { tx_hash: tx.tx_hash });
        }
        self.0.insert(tx.tx_hash, tx);

        Ok(())
    }

    pub fn remove(
        &mut self,
        tx_hash: &TransactionHash,
    ) -> Result<StagingTransaction, MempoolError> {
        match self.0.remove(tx_hash) {
            Some(tx) => Ok(tx),
            None => Err(MempoolError::TransactionNotFound { tx_hash: *tx_hash }),
        }
    }
}

impl From<ThinTransaction> for StagingTransaction {
    fn from(tx: ThinTransaction) -> Self {
        StagingTransaction { address: tx.sender_address, nonce: tx.nonce, tx_hash: tx.tx_hash }
    }
}
