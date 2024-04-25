use crate::{
    errors::MempoolError,
    priority_queue::{ContractAddressPriorityQueue, PriorityQueue},
};
use starknet_api::{
    core::{ContractAddress, Nonce},
    internal_transaction::InternalTransaction,
    transaction::TransactionHash,
};
use std::collections::HashMap;

#[cfg(test)]
#[path = "mempool_test.rs"]
pub mod mempool_test;

pub type MempoolResult<T> = Result<T, MempoolError>;

#[derive(Default)]
pub struct Mempool {
    priority_queue: PriorityQueue,
    contract_addresses_priority_queues: HashMap<ContractAddress, ContractAddressPriorityQueue>,
    state: HashMap<ContractAddress, Nonce>,
}

impl Mempool {
    pub fn new(inputs: Vec<MempoolInput>) -> Self {
        let mut mempool = Mempool::default();
        let mut new_txs = Vec::new();
        for input in inputs {
            mempool.state.insert(
                input.account_state.contract_address,
                input.account_state.nonce,
            );
            if let Some(contract_address_pq) = mempool
                .contract_addresses_priority_queues
                .get_mut(&input.account_state.contract_address)
            {
                // avoid duplicates.
                if !contract_address_pq.0.contains(&input.tx) {
                    contract_address_pq.push(input.tx.clone());
                    new_txs.push(input.tx);
                }
            } else {
                mempool.contract_addresses_priority_queues.insert(
                    input.account_state.contract_address,
                    ContractAddressPriorityQueue(vec![input.tx.clone()]),
                );
                new_txs.push(input.tx);
            }
        }
        mempool.priority_queue = PriorityQueue::from(new_txs);
        mempool
    }

    /// Retrieves up to `n_txs` transactions with the highest priority from the mempool.
    /// Transactions are guaranteed to be unique across calls until `commit_block` is invoked.
    // TODO: the last part about commit_block is incorrect if we delete txs in get_txs and then push back.
    pub fn get_txs(&mut self, n_txs: usize) -> MempoolResult<Vec<InternalTransaction>> {
        let txs = self.priority_queue.pop_last_chunk(n_txs);
        for tx in &txs {
            let contract_address = tx.contract_address();
            if let Some(contract_address_pq) = self
                .contract_addresses_priority_queues
                .get_mut(&contract_address)
            {
                self.state
                    .insert(contract_address, tx.nonce().try_increment()?);
                contract_address_pq.pop();
                if contract_address_pq.is_empty() {
                    self.contract_addresses_priority_queues
                        .remove(&contract_address);
                } else if let Some(next_tx) = contract_address_pq.top() {
                    self.priority_queue.push(next_tx);
                }
            }
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
        self.state
            .insert(account_state.contract_address, account_state.nonce);

        if let Some(contract_address_pq) = self
            .contract_addresses_priority_queues
            .get_mut(&account_state.contract_address)
        {
            if contract_address_pq.0.contains(&tx) {
                return Err(MempoolError::DuplicateTransaction);
            }
            contract_address_pq.0.push(tx);
        } else {
            self.contract_addresses_priority_queues.insert(
                account_state.contract_address,
                ContractAddressPriorityQueue(vec![tx.clone()]),
            );
            if self.state.get(&account_state.contract_address) == Some(&tx.nonce()) {
                self.priority_queue.push(tx);
            }
        }
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

#[derive(Clone, Default)]
pub struct AccountState {
    pub contract_address: ContractAddress,
    pub nonce: Nonce,
}

#[derive(Clone)]
pub struct MempoolInput {
    pub tx: InternalTransaction,
    pub account_state: AccountState,
}
