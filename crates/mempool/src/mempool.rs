use std::collections::HashMap;

use starknet_api::core::ContractAddress;
use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::{
    Account, AccountState, MempoolInput, MempoolResult, ThinTransaction,
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
    // TODO: If `n_txs` is greater than the number of transactions in `txs_queue`, it will also
    // check and add transactions from `address_to_store`.
    pub fn get_txs(&mut self, n_txs: usize) -> MempoolResult<Vec<ThinTransaction>> {
        let eligible_txs = self.tx_queue.pop_last_chunk(n_txs);
        // TODO: add staging area.
        // Remove transactions from mempool.
        for tx in &eligible_txs {
            if let Some(account_txs) = self.address_to_queue.get_mut(&tx.sender_address) {
                match account_txs.pop_front() {
                    Some(account_tx_ref) => assert_eq!(account_tx_ref.tx_hash, tx.tx_hash),
                    None => panic!("Queued transaction should be present in account transactions"),
                }
                match account_txs.top() {
                    Some(next_account_tx) => self.tx_queue.push(next_account_tx.clone()),
                    None => {
                        self.address_to_queue.remove(&tx.sender_address);
                    }
                }
            }
        }

        Ok(eligible_txs)
    }

    /// Adds a new transaction to the mempool.
    /// TODO: support fee escalation and transactions with future nonces.
    /// TODO: change input type to `MempoolInput`.
    pub fn add_tx(&mut self, tx: ThinTransaction, account: Account) -> MempoolResult<()> {
        if tx.nonce >= account.state.nonce {
            self.insert_tx(tx)?;
            Ok(())
        } else {
            Err(MempoolError::DuplicateTransaction { tx_hash: (tx.tx_hash) })
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

    fn insert_tx(&mut self, tx: ThinTransaction) -> MempoolResult<()> {
        match self.address_to_queue.get_mut(&tx.sender_address) {
            Some(address_queue) => {
                if address_queue.contains(&tx) {
                    return Err(MempoolError::DuplicateTransaction { tx_hash: tx.tx_hash });
                }
                address_queue.push(tx.clone());
            }
            None => {
                let mut address_queue = AddressPriorityQueue::default();
                address_queue.push(tx.clone());
                self.address_to_queue.insert(tx.sender_address, address_queue);
                self.tx_queue.push(tx);
            }
        }

        Ok(())
    }
}
