use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

use starknet_api::core::ContractAddress;
use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::{
    Account, AccountState, MempoolInput, MempoolResult, ThinTransaction,
};

use crate::priority_queue::TransactionPriorityQueue;
use crate::transaction_store::TransactionsStore;

#[cfg(test)]
#[path = "mempool_test.rs"]
pub mod mempool_test;

#[derive(Debug)]
pub struct Mempool {
    // TODO: add docstring explaining visibility and coupling of the fields.
    txs_queue: TransactionPriorityQueue,
    tx_store: TransactionsStore,
    state: HashMap<ContractAddress, AccountState>,
}

impl Mempool {
    // TODO(Mohammad): return `Result`, to consider invalid input.
    pub fn new(inputs: impl IntoIterator<Item = MempoolInput>) -> Self {
        let mut mempool = Mempool {
            txs_queue: TransactionPriorityQueue::default(),
            tx_store: TransactionsStore::default(),
            state: HashMap::default(),
        };

        mempool.txs_queue = TransactionPriorityQueue::from(
            inputs
                .into_iter()
                .map(|input| {
                    // Attempts to insert a key-value pair into the mempool's state. Returns `None`
                    // if the key was not present, otherwise returns the old value while updating
                    // the new value.
                    let prev_value =
                        mempool.state.insert(input.account.sender_address, input.account.state);
                    assert!(
                        prev_value.is_none(),
                        "Sender address: {:?} already exists in the mempool. Can't add {:?} to \
                         the mempool.",
                        input.account.sender_address,
                        input.tx
                    );

                    // Insert the transaction into the tx_store.
                    let res = mempool.tx_store.push(input.tx.clone());
                    assert!(
                        res.is_ok(),
                        "Transaction: {:?} already exists in the mempool.",
                        input.tx.tx_hash
                    );

                    input.tx
                })
                .collect::<Vec<ThinTransaction>>(),
        );

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
            self.state.remove(&tx.sender_address);
            self.tx_store.remove(&tx.tx_hash)?;
        }

        Ok(txs)
    }

    /// Adds a new transaction to the mempool.
    /// TODO: support fee escalation and transactions with future nonces.
    /// TODO: change input type to `MempoolInput`.
    pub fn add_tx(&mut self, tx: ThinTransaction, account: Account) -> MempoolResult<()> {
        match self.state.entry(account.sender_address) {
            Occupied(_) => Err(MempoolError::DuplicateTransaction { tx_hash: tx.tx_hash }),
            Vacant(entry) => {
                entry.insert(account.state);
                // TODO(Mohammad): use `handle_tx`.
                self.txs_queue.push(tx.clone());
                self.tx_store.push(tx)?;

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
