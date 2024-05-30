use std::collections::HashMap;

use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::transaction::{Tip, TransactionHash};
use starknet_mempool_types::mempool_types::{
    AccountState, MempoolInput, MempoolResult, ThinTransaction,
};

use crate::transaction_pool::TransactionPool;
use crate::transaction_queue::TransactionQueue;

#[cfg(test)]
#[path = "mempool_test.rs"]
pub mod mempool_test;

#[derive(Debug, Default)]
pub struct Mempool {
    // TODO: add docstring explaining visibility and coupling of the fields.
    // All transactions currently held in the mempool.
    tx_pool: TransactionPool,
    // Transactions eligible for sequencing.
    tx_queue: TransactionQueue,
    // Transactions that are currently being executed.
    staging: Vec<TransactionHash>,
}

impl Mempool {
    pub fn new(inputs: impl IntoIterator<Item = MempoolInput>) -> MempoolResult<Self> {
        let mut mempool = Mempool::empty();

        for input in inputs {
            mempool.insert_tx(input)?;
        }
        Ok(mempool)
    }

    pub fn empty() -> Self {
        Mempool::default()
    }

    /// Returns an iterator of the current eligible transactions for sequencing, ordered by their
    /// priority.
    pub fn iter(&self) -> impl Iterator<Item = &TransactionReference> {
        self.tx_queue.iter()
    }

    /// Retrieves up to `n_txs` transactions with the highest priority from the mempool.
    /// Transactions are guaranteed to be unique across calls until `commit_block` is invoked.
    // TODO: the last part about commit_block is incorrect if we delete txs in get_txs and then push
    // back. TODO: Consider renaming to `pop_txs` to be more consistent with the standard
    // library.
    pub fn get_txs(&mut self, n_txs: usize) -> MempoolResult<Vec<ThinTransaction>> {
        let mut eligible_txs: Vec<ThinTransaction> = Vec::with_capacity(n_txs);
        for tx_hash in self.tx_queue.pop_last_chunk(n_txs) {
            let tx = self.tx_pool.get(tx_hash)?.clone();
            assert!(!self.staging.contains(&tx_hash));
            self.staging.push(tx_hash);
            eligible_txs.push(tx);
        }

        Ok(eligible_txs)
    }

    // TODO(Ayelet): implement a method that returns the next eligible transaction for the given
    // sender address to be added to priority queue.
    #[allow(dead_code)]
    fn get_next_eligible_tx(
        &self,
        _sender_address: ContractAddress,
    ) -> Option<TransactionReference> {
        todo!()
    }

    /// Adds a new transaction to the mempool.
    /// TODO: support fee escalation and transactions with future nonces.
    /// TODO: check Account nonce and balance.
    pub fn add_tx(&mut self, input: MempoolInput) -> MempoolResult<()> {
        self.insert_tx(input)
    }

    /// Update the mempool's internal state according to the committed block's transactions.
    /// This method also updates internal state (resolves nonce gaps, updates account balances).
    // TODO: the part about resolving nonce gaps is incorrect if we delete txs in get_txs and then
    // push back.
    pub fn commit_block(
        &mut self,
        txs_in_block: &[TransactionHash],
        _state_changes: HashMap<ContractAddress, AccountState>,
    ) -> MempoolResult<()> {
        let mut counter = 0;
        for &tx_hash in txs_in_block {
            if self.staging.contains(&tx_hash) {
                counter += 1;
                self.tx_pool.remove(tx_hash)?;
            }
        }
        // It pops the first `counter` hashes from staging area.
        // Since transactions maintain their order after being processed by the Mempool, the
        // transactions to be included in the block should be the first ones in the staging area.
        self.staging.drain(0..counter);

        // Re-insert transaction to PQ.
        for &tx_hash in self.staging.iter() {
            let tx = self.tx_pool.get(tx_hash)?;
            self.tx_queue.insert(TransactionReference::new(tx));
        }

        // Cleanin the `StagingArea`.
        self.staging = Vec::default();

        Ok(())
    }

    fn insert_tx(&mut self, input: MempoolInput) -> MempoolResult<()> {
        let tx = input.tx;

        self.tx_pool.insert(tx.clone())?;
        self.tx_queue.insert(TransactionReference::new(&tx));

        Ok(())
    }
}

/// Provides a lightweight representation of a transaction for mempool usage (e.g., excluding
/// execution fields).
/// TODO(Mohammad): rename this struct to `ThinTransaction` once that name
/// becomes available, to better reflect its purpose and usage.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TransactionReference {
    pub sender_address: ContractAddress,
    pub nonce: Nonce,
    pub tx_hash: TransactionHash,
    pub tip: Tip,
}

impl TransactionReference {
    pub fn new(tx: &ThinTransaction) -> Self {
        TransactionReference {
            sender_address: tx.sender_address,
            nonce: tx.nonce,
            tx_hash: tx.tx_hash,
            tip: tx.tip,
        }
    }
}
