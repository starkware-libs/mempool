use starknet_api::{internal_transaction::InternalTransaction, transaction::TransactionHash};

use crate::{errors::MempoolError, priority_queue::PriorityQueue};

#[cfg(test)]
#[path = "mempool_test.rs"]
pub mod mempool_test;

pub type MempoolResult<T> = Result<T, MempoolError>;

pub struct Mempool {
    priority_queue: PriorityQueue,
}
impl Mempool {
    /// Retrieves up to `n_txs` transactions with the highest priority from the mempool.
    /// Transactions are guaranteed to be unique across calls until `commit_block` is invoked.
    // TODO: the last part about commit_block is incorrect if we delete txs in get_txs and then push back.
    pub fn get_tx(&mut self, n_txs: usize) -> MempoolResult<Vec<InternalTransaction>> {
        Ok(self.priority_queue.pop(n_txs))
    }

    /// Adds a new transaction to the mempool.
    /// TODO: support fee escalation and transactions with future nonces.
    pub fn add_tx(&mut self, _tx: InternalTransaction) -> MempoolResult<()> {
        todo!();
    }

    /// Update the mempool's internal state according to the committed block's transactions.
    /// This method also resolves nonce gaps and updates account balances.
    // TODO: the part about resolving nonce gaps is incorrect if we delete txs in get_txs and then
    // push back.
    pub fn commit_block(
        &mut self,
        _block_number: u64,
        _txs_in_block: &[TransactionHash],
        _state_changes: StateChanges,
    ) -> MempoolResult<()> {
        todo!()
    }
}

pub struct StateChanges;
