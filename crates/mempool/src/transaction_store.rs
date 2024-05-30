use std::collections::hash_map::Entry;
use std::collections::HashMap;

use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::ThinTransaction;

// All transactions currently held in the mempool.
#[derive(Clone, Debug, Default)]
pub struct TransactionStore {
    store: HashMap<TransactionHash, ThinTransaction>,
}

impl TransactionStore {
    pub fn push(&mut self, tx: ThinTransaction) -> Result<(), MempoolError> {
        match self.store.entry(tx.tx_hash) {
            Entry::Occupied(_) => {
                // TODO: Allow overriding a previous transaction if needed.
                Err(MempoolError::DuplicateTransaction { tx_hash: tx.tx_hash })
            }
            Entry::Vacant(entry) => {
                entry.insert(tx);
                Ok(())
            }
        }
    }

    pub fn remove(&mut self, tx_hash: &TransactionHash) -> Result<ThinTransaction, MempoolError> {
        self.store.remove(tx_hash).ok_or(MempoolError::TransactionNotFound { tx_hash: *tx_hash })
    }

    pub fn get(&self, tx_hash: &TransactionHash) -> Result<&ThinTransaction, MempoolError> {
        self.store.get(tx_hash).ok_or(MempoolError::TransactionNotFound { tx_hash: *tx_hash })
    }
}
