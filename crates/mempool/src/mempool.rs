use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

use async_trait::async_trait;
use mempool_infra::component_server::ComponentMessageExecutor;
use starknet_api::core::ContractAddress;
use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::{
    Account, AccountState, MempoolInput, MempoolMessages, MempoolResponses, MempoolResult,
    MempoolTrait, ThinTransaction,
};

use crate::priority_queue::PriorityQueue;

#[cfg(test)]
#[path = "mempool_test.rs"]
pub mod mempool_test;

pub struct Mempool {
    // TODO: add docstring explaining visibility and coupling of the fields.
    txs_queue: PriorityQueue,
    state: HashMap<ContractAddress, AccountState>,
}

impl Mempool {
    pub fn new(inputs: impl IntoIterator<Item = MempoolInput>) -> Self {
        let mut mempool =
            Mempool { txs_queue: PriorityQueue::default(), state: HashMap::default() };

        mempool.txs_queue = PriorityQueue::from_iter(inputs.into_iter().map(|input| {
            // Attempts to insert a key-value pair into the mempool's state. Returns `None` if the
            // key was not present, otherwise returns the old value while updating the new value.
            let prev_value = mempool.state.insert(input.account.address, input.account.state);
            // Assert that the contract address does not exist in the mempool's state to ensure that
            // there is only one transaction per contract address.
            assert!(
                prev_value.is_none(),
                "Contract address: {:?} already exists in the mempool. Can't add {:?} to the \
                 mempool.",
                input.account.address,
                input.tx
            );
            input.tx
        }));

        mempool
    }

    pub fn empty() -> Self {
        Mempool::new([])
    }

    /// Retrieves up to `n_txs` transactions with the highest priority from the mempool.
    /// Transactions are guaranteed to be unique across calls until `commit_block` is invoked.
    // TODO: the last part about commit_block is incorrect if we delete txs in get_txs and then push
    // back. TODO: Consider renaming to `pop_txs` to be more consistent with the standard
    // library.
    pub fn get_txs(&mut self, n_txs: usize) -> MempoolResult<Vec<ThinTransaction>> {
        let txs = self.txs_queue.pop_last_chunk(n_txs);
        for tx in &txs {
            self.state.remove(&tx.contract_address);
        }
        Ok(txs)
    }

    /// Adds a new transaction to the mempool.
    /// TODO: support fee escalation and transactions with future nonces.
    /// TODO: change input type to `MempoolInput`.
    pub fn add_tx(&mut self, tx: ThinTransaction, account: Account) -> MempoolResult<()> {
        match self.state.entry(account.address) {
            Occupied(_) => Err(MempoolError::DuplicateTransaction { tx_hash: tx.tx_hash }),
            Vacant(entry) => {
                entry.insert(account.state);
                self.txs_queue.push(tx);
                Ok(())
            }
        }
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

#[async_trait]
impl MempoolTrait for Mempool {
    async fn async_add_tx(&mut self, tx: ThinTransaction, account: Account) -> MempoolResult<()> {
        self.add_tx(tx, account)
    }

    async fn async_get_txs(&mut self, n_txs: usize) -> MempoolResult<Vec<ThinTransaction>> {
        self.get_txs(n_txs)
    }
}

#[async_trait]
impl ComponentMessageExecutor<MempoolMessages, MempoolResponses> for Mempool {
    async fn execute(&mut self, message: MempoolMessages) -> MempoolResponses {
        match message {
            MempoolMessages::AsyncAddTransaction(tx, account) => {
                let res = self.async_add_tx(tx, account).await;
                MempoolResponses::AsyncAddTransaction(res)
            }
            MempoolMessages::AsyncGetTxs(n_txs) => {
                let res = self.async_get_txs(n_txs).await;
                MempoolResponses::AsyncGetTxs(res)
            }
        }
    }
}
