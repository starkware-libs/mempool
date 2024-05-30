use std::collections::hash_map::Entry;
use std::collections::{btree_map, BTreeMap, HashMap};

use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::{MempoolResult, ThinTransaction};

use crate::priority_queue::PrioritizedTransaction;

/// Contains all transactions currently held in the mempool.
/// Invariant: both data structures are consistent regarding the existence of transactions:
/// A transaction appears in one if and only if it appears in the other.
/// No duplicate transactions appear in the pool.
#[derive(Clone, Debug, Default)]
pub struct TransactionsStore {
    // Holds the complete transaction objects; it should be the sole entity that does so.
    store: HashMap<TransactionHash, ThinTransaction>,
    // Transactions organized by account address, sorted by ascending nonce values.
    txs_by_account: HashMap<ContractAddress, BTreeMap<Nonce, PrioritizedTransaction>>,
    // Invariants:
    // 1. Every transaction in `txs_by_account` must have a corresponding entry in `store`.
    // 2. When a transaction is added to `store`, it must also be added to `txs_by_account`.
    // 3. When a transaction is removed from `store`, it must also be removed from
    //    `txs_by_account`.
}

impl TransactionsStore {
    pub fn push(&mut self, tx: ThinTransaction) -> MempoolResult<()> {
        match self.store.entry(tx.tx_hash) {
            Entry::Vacant(entry) => {
                entry.insert(tx.clone());
            }
            Entry::Occupied(_) => {
                return Err(MempoolError::DuplicateTransaction { tx_hash: tx.tx_hash });
            }
        }

        match self.txs_by_account.entry(tx.sender_address).or_default().entry(tx.nonce) {
            btree_map::Entry::Vacant(entry) => {
                entry.insert(tx.into());
            }
            btree_map::Entry::Occupied(_) => {
                panic!("Consistency error: {tx:?} wasn't in storage but is in account storage")
            }
        }
        Ok(())
    }

    pub fn remove(&mut self, tx_hash: &TransactionHash) -> Result<ThinTransaction, MempoolError> {
        let tx = self
            .store
            .remove(tx_hash)
            .ok_or(MempoolError::TransactionNotFound { tx_hash: *tx_hash })?;

        match self.txs_by_account.entry(tx.sender_address) {
            Entry::Occupied(mut entry) => {
                let txs_by_account = entry.get_mut();
                assert!(
                    txs_by_account.remove(&tx.nonce).is_some(),
                    "Invariant violated: Trying to remove a transaction that does not exist in \
                     txs_by_account."
                );
            }
            Entry::Vacant(_) => panic!(
                "Invariant violated: Trying to remove a transaction that does not exist in \
                 txs_by_account."
            ),
        }
        Ok(tx)
    }

    pub fn get(&self, tx_hash: &TransactionHash) -> Result<&ThinTransaction, MempoolError> {
        match self.store.get(tx_hash) {
            Some(tx) => Ok(tx),
            None => Err(MempoolError::TransactionNotFound { tx_hash: *tx_hash }),
        }
    }
}
