use std::collections::HashMap;

use starknet_api::{
    core::{ContractAddress, Nonce},
    internal_transaction::InternalTransaction,
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
    pub fn new(inputs: Vec<MempoolInput>) -> Self {
        let mut mempool = Mempool::default();
        let mut new_transactions = Vec::new();
        for input in inputs {
            let account_state = &input.account_state;
            let tx = input.tx;
            if mempool.state.get(&account_state.contract_address).is_none() {
                mempool
                    .state
                    .insert(account_state.contract_address, account_state.nonce);
                new_transactions.push(tx);
            }
        }
        mempool.priority_queue = PriorityQueue::from(new_transactions);
        mempool
    }

    /// Retrieves up to `n_txs` transactions with the highest priority from the mempool.
    /// Transactions are guaranteed to be unique across calls until `commit_block` is invoked.
    // TODO: the last part about commit_block is incorrect if we delete txs in get_txs and then push back.
    pub fn get_txs(&mut self, n_txs: usize) -> MempoolResult<Vec<InternalTransaction>> {
        let txs = self.priority_queue.pop_last_chunk(n_txs);
        for tx in &txs {
            self.state.remove(&tx.contract_address());
        }
        Ok(txs)
    }

    /// Adds a new transaction to the mempool.
    /// TODO: support fee escalation and transactions with future nonces.
    pub fn add_tx(
        &mut self,
        _tx: InternalTransaction,
        _account_state: AccountState,
    ) -> MempoolResult<()> {
        todo!();
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

#[derive(Default, Clone)]
pub struct AccountState {
    pub contract_address: ContractAddress,
    pub nonce: Nonce,
}

pub struct MempoolInput {
    pub tx: InternalTransaction,
    pub account_state: AccountState,
}
