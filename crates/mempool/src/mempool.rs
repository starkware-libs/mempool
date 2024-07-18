use std::collections::HashMap;

use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::transaction::{Tip, TransactionHash};
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::{
    Account, AccountState, MempoolInput, MempoolResult, ThinTransaction,
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
    // Represents the current state of the mempool during block creation.
    mempool_state: HashMap<ContractAddress, AccountState>,
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
        for tx_hash in self.tx_queue.pop_chunk(n_txs) {
            let tx = self.tx_pool.remove(tx_hash)?;
            eligible_txs.push(tx);
        }

        // Update the mempool state with the new nonces.
        for tx in &eligible_txs {
            self.mempool_state.entry(tx.sender_address).or_default().nonce = tx.nonce;
        }

        Ok(eligible_txs)
    }

    /// Adds a new transaction to the mempool.
    /// TODO: support fee escalation and transactions with future nonces.
    /// TODO: check Account nonce and balance.
    pub fn add_tx(&mut self, input: MempoolInput) -> MempoolResult<()> {
        self.is_duplicated_tx(&input.tx)?;
        self.insert_tx(input)
    }

    /// Update the mempool's internal state according to the committed block (resolves nonce gaps,
    /// updates account balances).
    // TODO: the part about resolving nonce gaps is incorrect if we delete txs in get_txs and then
    // push back.
    // state_changes: a map that associates each account address with the state of the committed
    // block.
    pub fn commit_block(
        &mut self,
        state_changes: HashMap<ContractAddress, AccountState>,
    ) -> MempoolResult<()> {
        for (address, AccountState { nonce }) in state_changes {
            let next_nonce = nonce.try_increment().map_err(|_| MempoolError::FeltOutOfRange)?;
            // Dequeue transactions from the queue in the following cases:
            // 1. Remove a transaction from queue with nonce lower and eq than those committed to
            //    the block, applicable when the block is from the same leader.
            // 2. Remove a transaction from queue with nonce greater than the next nonce block,
            //    applicable when the block is from a different leader.
            if self
                .tx_queue
                .get_nonce(address)
                .is_some_and(|queued_nonce| queued_nonce != next_nonce)
            {
                self.tx_queue.remove(address);
            }
            // TODO: remove the transactions from the tx_pool.
        }
        // TODO: update the tx_queue with the new state changes.
        todo!()
    }

    fn insert_tx(&mut self, input: MempoolInput) -> MempoolResult<()> {
        let MempoolInput { tx, account } = input;
        let tx_reference = TransactionReference::new(&tx);

        self.tx_pool.insert(tx)?;

        if is_eligible_for_sequencing(tx_reference, account) {
            self.tx_queue.insert(tx_reference);
        }

        Ok(())
    }

    fn is_duplicated_tx(&self, tx: &ThinTransaction) -> MempoolResult<()> {
        if let Some(AccountState { nonce }) = self.mempool_state.get(&tx.sender_address) {
            if nonce >= &tx.nonce {
                return Err(MempoolError::DuplicateTransaction { tx_hash: tx.tx_hash });
            }
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn _tx_pool(&self) -> &TransactionPool {
        &self.tx_pool
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

fn is_eligible_for_sequencing(tx_reference: TransactionReference, account: Account) -> bool {
    tx_reference.nonce == account.state.nonce
}
