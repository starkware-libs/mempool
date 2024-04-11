use std::collections::HashMap;

use starknet_api::{
    core::ContractAddress, core::Nonce, internal_transaction::InternalTransaction,
    transaction::TransactionHash,
};

use crate::{errors::MempoolError, priority_queue::PriorityQueue};

#[cfg(test)]
#[path = "mempool_test.rs"]
pub mod mempool_test;

pub type MempoolResult<T> = Result<T, MempoolError>;

#[derive(Default)]
pub struct Mempool {
    priority_queue: PriorityQueue,
    state: HashMap<ContractAddress, Nonce>,
}

impl Mempool {
    /// Retrieves up to `n_txs` transactions with the highest priority from the mempool.
    /// Transactions are guaranteed to be unique across calls until `commit_block` is invoked.
    // TODO: the last part about commit_block is incorrect if we delete txs in get_txs and then push back.
    pub fn get_txs(&mut self, n_txs: usize) -> MempoolResult<Vec<InternalTransaction>> {
        let txs = self.priority_queue.split_off(n_txs);
        for tx in &txs {
            self.state.remove(&tx.contract_address());
        }
        Ok(txs)
    }

    /// Adds a new transaction to the mempool.
    /// TODO: support fee escalation and transactions with future nonces.
    pub fn add_tx(
        &mut self,
        tx: InternalTransaction,
        account_state: &AccountState,
    ) -> MempoolResult<()> {
        if self.state.contains_key(&account_state.contract_address) {
            return Err(MempoolError::DuplicateTransaction);
        }
        self.state
            .insert(account_state.contract_address, account_state.nonce);
        self.priority_queue.push(tx);
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

#[derive(Default)]
pub struct AccountState {
    pub contract_address: ContractAddress,
    pub nonce: Nonce,
}
