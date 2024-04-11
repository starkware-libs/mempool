use starknet_api::{internal_transaction::InternalTransaction, transaction::TransactionHash};

use crate::errors::MempoolError;

pub type MempoolResult<T> = Result<T, MempoolError>;

pub struct Mempool;

impl Mempool {
    /// Retrieves up to `n_txs` transactions with the highest priority from the mempool.
    /// Transactions are guaranteed to be unique across calls until `commit_block` is invoked.
    // TODO: the last part about commit_block is incorrect if we delete txs in get_txs and then push back.
    pub fn get_txs(n_txs: u8) -> MempoolResult<Vec<InternalTransaction>> {
        todo!();
    }

    /// Adds a new transaction to the mempool.
    /// TODO: support fee escalation and transactions with future nonces.
    pub fn add_tx(&mut self, tx: InternalTransaction) -> MempoolResult<()> {
        todo!();
    }

    /// Update the mempool's internal state according to the committed block's transactions.
    /// This method also resolves nonce gaps and updates account balances.
    // TODO: the part about resolving nonce gaps is incorrect if we delete txs in get_txs and then
    // push back.
    pub fn commit_block(
        &mut self,
        block_number: u64,
        txs_in_block: &[TransactionHash],
        state_changes: StateChanges,
    ) -> MempoolResult<()> {
        todo!()
    }
}

pub struct StateChanges;
