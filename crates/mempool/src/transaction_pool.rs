use std::collections::{hash_map, BTreeMap, HashMap};

use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::{MempoolResult, ThinTransaction};

use crate::mempool::TransactionReference;

/// Contains all transactions currently held in the mempool.
/// Invariant: both data structures are consistent regarding the existence of transactions:
/// A transaction appears in one if and only if it appears in the other.
/// No duplicate transactions appear in the pool.
#[derive(Clone, Debug, Default)]
pub struct TransactionPool {
    // Holds the complete transaction objects; it should be the sole entity that does so.
    tx_pool: HashMap<TransactionHash, ThinTransaction>,
    // Transactions organized by account address, sorted by ascending nonce values.
    txs_by_account: AccountTransactionIndex,
}

impl TransactionPool {
    // TODO(Mohammad): Remove the cloning of tx once the `TransactionReference` is updated.
    pub fn insert(&mut self, tx: ThinTransaction) -> MempoolResult<()> {
        let tx_hash = tx.tx_hash;

        // Insert transaction to pool, if it is new.
        if let hash_map::Entry::Vacant(entry) = self.tx_pool.entry(tx_hash) {
            entry.insert(tx.clone());
        } else {
            return Err(MempoolError::DuplicateTransaction { tx_hash });
        }

        let unexpected_existing_tx = self.txs_by_account.insert(TransactionReference::new(tx));
        if unexpected_existing_tx.is_some() {
            panic!(
                "Transaction pool consistency error: transaction with hash {tx_hash} does not \
                 appear in main mapping, but it appears in the account mapping",
            )
        };

        Ok(())
    }

    pub fn remove(&mut self, tx_hash: TransactionHash) -> MempoolResult<ThinTransaction> {
        let tx =
            self.tx_pool.remove(&tx_hash).ok_or(MempoolError::TransactionNotFound { tx_hash })?;

        self
            .txs_by_account
            // FIXME: remove clone once TransactionReference has a constructor from a refernece.
            .remove(TransactionReference::new(tx.clone()))
            .unwrap_or_else(|| {
            panic!(
                "Transaction pool consistency error: transaction with hash {tx_hash} appears in \
                 main mapping, but does not appear in the account mapping"
            )
        });

        Ok(tx)
    }

    pub fn get(&self, tx_hash: TransactionHash) -> MempoolResult<&ThinTransaction> {
        self.tx_pool.get(&tx_hash).ok_or(MempoolError::TransactionNotFound { tx_hash })
    }
}

// TODO: Use in txs_by_account.
// TODO: remove when is used.
#[derive(Debug, Default)]
pub struct AccountTransactionIndex(
    pub HashMap<ContractAddress, BTreeMap<Nonce, TransactionReference>>,
);

impl AccountTransactionIndex {
    /// If the transaction already exists in the mapping, the old value is returned.
    pub fn insert(&mut self, tx: TransactionReference) -> Option<TransactionReference> {
        self.0.entry(tx.sender_address).or_default().insert(tx.nonce, tx)
    }

    pub fn remove(&mut self, tx: TransactionReference) -> Option<TransactionReference> {
        let ThinTransaction { sender_address, nonce, .. } = tx.0;
        let account_txs = self.0.get_mut(&sender_address)?;

        let removed = account_txs.remove(&nonce);

        if removed.is_some() && account_txs.is_empty() {
            self.0.remove(&sender_address);
        }

        removed
    }
}
