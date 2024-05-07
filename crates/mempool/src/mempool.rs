use tokio::{select, sync::Notify};

use mempool_infra::network_component::CommunicationInterface;
use std::{collections::HashMap, sync::Arc};

use crate::{errors::MempoolError, priority_queue::PriorityQueue};
use starknet_api::{
    core::ContractAddress, internal_transaction::InternalTransaction, transaction::TransactionHash,
};
use std::collections::hash_map::Entry::{Occupied, Vacant};

#[cfg(test)]
#[path = "mempool_test.rs"]
pub mod mempool_test;

use starknet_mempool_types::mempool_types::{
    Account, AccountState, GatewayToMempoolMessage, MempoolInput, MempoolNetworkComponent,
};

pub type MempoolResult<T> = Result<T, MempoolError>;

pub struct Mempool {
    // TODO: add docstring explaining visibility and coupling of the fields.
    pub network: MempoolNetworkComponent,
    txs_queue: PriorityQueue,
    state: HashMap<ContractAddress, AccountState>,
}

impl Mempool {
    pub fn new(
        inputs: impl IntoIterator<Item = MempoolInput>,
        network: MempoolNetworkComponent,
    ) -> Self {
        let mut mempool = Mempool {
            txs_queue: Default::default(),
            state: Default::default(),
            network,
        };

        mempool.txs_queue = PriorityQueue::from_iter(inputs.into_iter().map(|input| {
            // Attempts to insert a key-value pair into the mempool's state. Returns `None` if the
            // key was not present, otherwise returns the old value while updating the new value.
            let prev_value = mempool.state.insert(input.account.address, input.account.state);
            // Assert that the contract address does not exist in the mempool's state to ensure that
            // there is only one transaction per contract address.
            assert!(
                prev_value.is_none(),
                "Contract address: {:?} already exists in the mempool. Can't add {:?} to the mempool.",
                input.account.address, input.tx
            );
            input.tx
        }));

        mempool
    }

    /// Retrieves up to `n_txs` transactions with the highest priority from the mempool.
    /// Transactions are guaranteed to be unique across calls until `commit_block` is invoked.
    // TODO: the last part about commit_block is incorrect if we delete txs in get_txs and then push back.
    // TODO: Consider renaming to `pop_txs` to be more consistent with the standard library.
    pub fn get_txs(&mut self, n_txs: usize) -> MempoolResult<Vec<InternalTransaction>> {
        let txs = self.txs_queue.pop_last_chunk(n_txs);
        for tx in &txs {
            self.state.remove(&tx.contract_address());
        }

        Ok(txs)
    }

    /// Adds a new transaction to the mempool.
    /// TODO: support fee escalation and transactions with future nonces.
    pub fn add_tx(&mut self, tx: InternalTransaction, account: Account) -> MempoolResult<()> {
        match self.state.entry(account.address) {
            Occupied(_) => Err(MempoolError::DuplicateTransaction {
                tx_hash: tx.tx_hash(),
            }),
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

    /// Starts an asynchronous task that listens for network messages and processes them.
    pub async fn start_network_listener(&mut self, notify: Arc<Notify>) {
        loop {
            select! {
                message = self.network.recv() => {
                    match message {
                        Some(msg) => {
                            // Process the message
                            self.process_network_message(msg).await;
                        },
                        None => break, // Channel is closed, so exit the loop
                    }
                }
                _ = notify.notified() => {
                    // If notified, break the loop regardless of message state
                    break;
                }
            }
        }
    }

    /// Processes a single message received from the network.
    async fn process_network_message(&mut self, message: GatewayToMempoolMessage) {
        // Process the message
        // Example processing
        match message {
            GatewayToMempoolMessage::AddTx(tx, account_state) => {
                // Handle new transaction
                let _ = self.add_tx(tx, account_state);
            }
        }
    }
}
