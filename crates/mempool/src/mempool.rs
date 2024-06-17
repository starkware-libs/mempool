use std::collections::HashMap;

use starknet_api::core::ContractAddress;
use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::{
    AccountState, MempoolInput, MempoolResult, ThinTransaction,
};

use crate::priority_queue::{AddressPriorityQueue, TransactionPriorityQueue};

#[cfg(test)]
#[path = "mempool_test.rs"]
pub mod mempool_test;

#[derive(Default)]
pub struct Mempool {
    tx_queue: TransactionPriorityQueue,
    address_to_queue: HashMap<ContractAddress, AddressPriorityQueue>,
}

impl Mempool {
    pub fn new(inputs: impl IntoIterator<Item = MempoolInput>) -> MempoolResult<Self> {
        let mut mempool = Mempool::default();

        for input in inputs {
            mempool.insert_tx(input.tx)?;
        }

        Ok(mempool)
    }

    pub fn empty() -> Self {
        Mempool::default()
    }

    /// Retrieves up to `n_txs` transactions with the highest priority from the mempool.
    /// Transactions are guaranteed to be unique across calls until `commit_block` is invoked.
    // TODO: the last part about commit_block is incorrect if we delete txs in get_txs and then push
    // back.
    // TODO: Consider renaming to `pop_txs` to be more consistent with the standard library.
    pub fn get_txs(&mut self, n_txs: usize) -> MempoolResult<Vec<ThinTransaction>> {
        let mut eligible_txs = Vec::new();
        let mut remaining_txs = n_txs;

        while remaining_txs > 0 {
            let chunk = self.tx_queue.pop_last_chunk(remaining_txs);
            if chunk.is_empty() {
                break;
            }

            for tx in &chunk {
                // TODO: add staging area.
                if let Some(next_tx) = self.get_next_eligible_tx(tx.sender_address) {
                    self.tx_queue.push(next_tx);
                }
            }

            remaining_txs -= chunk.len();
            eligible_txs.extend(chunk);
        }

        Ok(eligible_txs)
    }

    /// Adds a new transaction to the mempool.
    /// TODO: support fee escalation and transactions with future nonces.
    /// TODO: change input type to `MempoolInput`.
    pub fn add_tx(&mut self, tx: ThinTransaction) -> MempoolResult<()> {
        self.insert_tx(tx)?;
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

    fn insert_tx(&mut self, tx: ThinTransaction) -> MempoolResult<()> {
        let address_queue = self.address_to_queue.entry(tx.sender_address).or_default();

        if address_queue.contains(&tx) {
            return Err(MempoolError::DuplicateTransaction { tx_hash: tx.tx_hash });
        }

        address_queue.push(tx.clone());
        if address_queue.len() == 1 {
            self.tx_queue.push(tx);
        }

        Ok(())
    }

    fn get_next_eligible_tx(&mut self, sender_address: ContractAddress) -> Option<ThinTransaction> {
        // Remove transactions from address priority queue.
        if let Some(address_queue) = self.address_to_queue.get_mut(&sender_address) {
            address_queue.pop_front();

            // Remove address from the mempool if no transactions from this account left.
            if address_queue.is_empty() {
                self.address_to_queue.remove(&sender_address);
            // Return next nonce transaction.
            } else {
                return address_queue.pop_front();
            }
        }
        None
    }
}
