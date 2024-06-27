use std::collections::HashMap;

use starknet_api::core::ContractAddress;
use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::mempool_types::{
    Account, AccountState, MempoolInput, MempoolResult, ThinTransaction,
};

use crate::priority_queue::TransactionQueue;
use crate::transaction_pool::TransactionPool;

#[cfg(test)]
#[path = "mempool_test.rs"]
pub mod mempool_test;

#[derive(Debug, Default)]
pub struct Mempool {
    // TODO: add docstring explaining visibility and coupling of the fields.
    txs_queue: TransactionQueue,
    tx_pool: TransactionPool,
}

impl Mempool {
    pub fn new(inputs: impl IntoIterator<Item = MempoolInput>) -> Self {
        let mut mempool = Mempool::empty();

        for MempoolInput { tx, .. } in inputs.into_iter() {
            // Attempt to push the transaction into the tx_pool
            if let Err(err) = mempool.tx_pool.push(tx.clone()) {
                panic!(
                    "Transaction: {:?} already exists in the mempool. Error: {:?}",
                    tx.tx_hash, err
                );
            }

            mempool.txs_queue.push(tx.into());
        }

        mempool
    }

    pub fn empty() -> Self {
        Mempool::default()
    }

    /// Retrieves up to `n_txs` transactions with the highest priority from the mempool.
    /// Transactions are guaranteed to be unique across calls until `commit_block` is invoked.
    // TODO: the last part about commit_block is incorrect if we delete txs in get_txs and then push
    // back. TODO: Consider renaming to `pop_txs` to be more consistent with the standard
    // library.
    pub fn get_txs(&mut self, n_txs: usize) -> MempoolResult<Vec<ThinTransaction>> {
        let mut eligible_txs: Vec<ThinTransaction> = Vec::with_capacity(n_txs);

        let txs = self.txs_queue.pop_last_chunk(n_txs);
        for tx in txs {
            self.tx_pool.remove(tx.tx_hash)?;
            eligible_txs.push(tx.0);
        }

        Ok(eligible_txs)
    }

    /// Adds a new transaction to the mempool.
    /// TODO: support fee escalation and transactions with future nonces.
    /// TODO: change input type to `MempoolInput`.
    pub fn add_tx(&mut self, tx: ThinTransaction, _account: Account) -> MempoolResult<()> {
        // TODO(Mohammad): use `handle_tx`.
        self.txs_queue.push(tx.clone().into());
        self.tx_pool.push(tx)?;

        Ok(())
    }

    /// Update the mempool's internal state according to the committed block's transactions.
    /// This method also updates internal state (resolves nonce gaps, updates account balances).
    // TODO: the part about resolving nonce gaps is incorrect if we delete txs in get_txs and then
    // push back.
    pub fn commit_block(
        &mut self,
        _block_number: u64,
        _txs_in_block: &[TransactionHash],
        _state_changes: HashMap<ContractAddress, AccountState>,
    ) -> MempoolResult<()> {
        todo!()
    }
}

/// Provides a lightweight representation of a transaction for mempool usage (e.g., excluding
/// execution fields).
/// TODO(Mohammad): rename this struct to `ThinTransaction` once that name
/// becomes available, to better reflect its purpose and usage.
#[derive(Clone, Debug, Default, derive_more::Deref, derive_more::From)]
pub struct TransactionReference(ThinTransaction);
