use std::collections::HashMap;

use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::transaction::{Tip, TransactionHash};
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::{
    Account, AccountState, MempoolInput, MempoolResult, ThinTransaction,
};
use starknet_types_core::felt::Felt;

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

        Ok(eligible_txs)
    }

    /// Adds a new transaction to the mempool.
    /// TODO: support fee escalation and transactions with future nonces.
    /// TODO: check Account nonce and balance.
    pub fn add_tx(&mut self, input: MempoolInput) -> MempoolResult<()> {
        // TODO(Yael 8/7/2024): Consider removing this check and instead add an API for the gateway
        // to check if the deploy_account tx exists.
        self.should_insert(&input)?;
        self.insert_tx(input)
    }

    /// Update the mempool's internal state according to the committed block (resolves nonce gaps,
    /// updates account balances).
    // TODO: the part about resolving nonce gaps is incorrect if we delete txs in get_txs and then
    // push back.
    pub fn commit_block(
        &mut self,
        state_changes: HashMap<ContractAddress, AccountState>,
    ) -> MempoolResult<()> {
        for (address, AccountState { nonce }) in state_changes {
            // Dequeue transactions from the queue in the following cases:
            // 1. Remove a transaction from queue with nonce lower than those committed to the
            //    block, applicable when the block is from the same leader.
            // 2. Remove a transaction from queue with nonce greater than those committed to the
            //    block, applicable when the block is from a different leader.
            if self.tx_queue.get_nonce(address).is_some_and(|queued_nonce| queued_nonce != nonce) {
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

    pub fn should_insert(&self, input: &MempoolInput) -> MempoolResult<()> {
        // If the tx nonce is 1 and the account nonce is 0, the account was not deployed yet and the
        // gateway has skipped validations. In this case, we need to verify that a deploy_account
        // transaction exists for this account. It is suficient to check if the account exists in
        // the mempool since it means that either it has a deploy_account transaction or
        // transactions with future nonces that passed validations.
        // TODO(Yael 8/7/2024): Consider instead of checking the nonces, get a value from the
        // gateway that indicates that the mempool needs to check the deploy_account existence.
        if input.tx.nonce == Nonce(Felt::ONE)
            && input.account.state.nonce == Nonce(Felt::ZERO)
            && !self.tx_pool.contains_address(input.tx.sender_address)
        {
            return Err(MempoolError::UndeployedAccount {
                sender_address: input.tx.sender_address,
            });
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
