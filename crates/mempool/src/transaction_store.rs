use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};

use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::ThinTransaction;

use crate::priority_queue::PrioritizedTransaction;

#[derive(Clone, Debug, Default)]
pub struct TransactionStore {
    // All transactions currently held in the mempool.
    store: HashMap<TransactionHash, ThinTransaction>,
    // Transactions organized by account address, sorted by ascending nonce values.
    txs_by_account: HashMap<ContractAddress, BTreeMap<Nonce, PrioritizedTransaction>>,
}

impl TransactionStore {
    // Insert transaction into the store, ensuring no duplicates
    pub fn push(&mut self, tx: ThinTransaction) -> Result<(), MempoolError> {
        if let Entry::Occupied(_) = self.store.entry(tx.tx_hash) {
            return Err(MempoolError::DuplicateTransaction { tx_hash: tx.tx_hash });
        } else {
            self.store.insert(tx.tx_hash, tx.clone());
        }

        match self.txs_by_account.entry(tx.sender_address) {
            Entry::Occupied(mut entry) => {
                let txs_by_account = entry.get_mut();
                if txs_by_account.contains_key(&tx.nonce) {
                    // Remove the transaction from the store if duplicate nonce found
                    self.store.remove(&tx.tx_hash);
                    return Err(MempoolError::DuplicateTransaction { tx_hash: tx.tx_hash });
                }
                txs_by_account.insert(tx.nonce, tx.into());
            }
            Entry::Vacant(entry) => {
                let mut txs_by_account = BTreeMap::new();
                txs_by_account.insert(tx.nonce, tx.into());
                entry.insert(txs_by_account);
            }
        }

        Ok(())
    }

    pub fn remove(&mut self, tx_hash: &TransactionHash) -> Result<ThinTransaction, MempoolError> {
        // Remove the transaction from the store
        let tx = self.store.remove(tx_hash);

        if tx.is_none() {
            return Err(MempoolError::TransactionNotFound { tx_hash: *tx_hash });
        }
        let tx = tx.unwrap();

        if let Entry::Occupied(mut entry) = self.txs_by_account.entry(tx.sender_address) {
            let txs_by_account = entry.get_mut();
            txs_by_account.remove(&tx.nonce);

            if txs_by_account.is_empty() {
                entry.remove();
            }
            Ok(tx)
        } else {
            Err(MempoolError::TransactionNotFound { tx_hash: tx.tx_hash })
        }
    }

    pub fn get(&self, tx_hash: &TransactionHash) -> Result<&ThinTransaction, MempoolError> {
        match self.store.get(tx_hash) {
            Some(tx) => Ok(tx),
            None => Err(MempoolError::TransactionNotFound { tx_hash: *tx_hash }),
        }
    }
}
