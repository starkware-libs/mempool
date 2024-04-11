use std::collections::HashMap;

use crate::{errors::MempoolError, priority_queue::PriorityQueue};
use starknet_api::{
    core::{ContractAddress, Nonce},
    internal_transaction::InternalTransaction,
    transaction::TransactionHash,
};

#[cfg(test)]
#[path = "mempool_test.rs"]
pub mod mempool_test;

pub type MempoolResult<T> = Result<T, MempoolError>;

#[derive(Default)]
pub struct Mempool {
    // TODO: add docstring explaining visibility and coupling of the fields.
    txs_queue: PriorityQueue,
    state: HashMap<ContractAddress, AccountState>,
}

impl Mempool {
    pub fn new<I>(inputs: I) -> Self
    where
        I: IntoIterator<Item = MempoolInput>,
    {
        let mut mempool = Mempool::default();

        mempool.txs_queue = PriorityQueue::from_iter(inputs.into_iter().map(|input| {
            // Assert that the contract address does not exist in the mempool's state to ensure that
            // there is only one transaction per contract address.
            assert!(
                mempool.state.get(&input.account.address).is_none(),
                "Contract address is already in the mempool"
            );
            mempool
                .state
                .insert(input.account.address, input.account.state);
            input.tx
        }));

        mempool
    }

    /// Retrieves up to `n_txs` transactions with the highest priority from the mempool.
    /// Transactions are guaranteed to be unique across calls until `commit_block` is invoked.
    // TODO: the last part about commit_block is incorrect if we delete txs in get_txs and then push back.
    pub fn get_txs(&mut self, n_txs: usize) -> MempoolResult<Vec<InternalTransaction>> {
        let txs = self.txs_queue.pop_last_chunk(n_txs);
        for tx in &txs {
            self.state.remove(&tx.contract_address());
        }

        Ok(txs)
    }

    /// Adds a new transaction to the mempool.
    /// TODO: support fee escalation and transactions with future nonces.
    pub fn add_tx(&mut self, tx: InternalTransaction, account: &Account) -> MempoolResult<()> {
        if self.state.contains_key(&account.address) {
            return Err(MempoolError::DuplicateTransaction {
                tx_hash: tx.tx_hash(),
            });
        }
        self.state.insert(account.address, account.state);
        self.txs_queue.push(tx);

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
#[derive(Clone, Copy, Default)]
pub struct AccountState {
    pub nonce: Nonce,
    // TODO: add balance field when needed.
}

#[derive(Default)]
pub struct Account {
    pub address: ContractAddress,
    pub state: AccountState,
}

pub struct MempoolInput {
    pub tx: InternalTransaction,
    pub account: Account,
}
