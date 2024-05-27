use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use mempool_infra::component_server::ComponentMessageExecutor;
use mempool_infra::network_component::CommunicationInterface;
use starknet_api::core::ContractAddress;
use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::{
    Account, AccountState, BatcherToMempoolChannels, BatcherToMempoolMessage,
    GatewayToMempoolMessage, MempoolInput, MempoolInterface, MempoolInvocationMessages,
    MempoolInvocationResponses, MempoolNetworkComponent, MempoolResult, ThinTransaction,
};
use tokio::select;
use tokio::sync::Mutex;

use crate::priority_queue::TransactionPriorityQueue;

#[cfg(test)]
#[path = "mempool_test.rs"]
pub mod mempool_test;

pub struct Mempool {
    // TODO: add docstring explaining visibility and coupling of the fields.
    pub gateway_network: MempoolNetworkComponent,
    batcher_network: BatcherToMempoolChannels,
    txs_queue: TransactionPriorityQueue,
    state: HashMap<ContractAddress, AccountState>,
}

impl Mempool {
    pub fn new(
        inputs: impl IntoIterator<Item = MempoolInput>,
        gateway_network: MempoolNetworkComponent,
        batcher_network: BatcherToMempoolChannels,
    ) -> Self {
        let mut mempool = Mempool {
            txs_queue: TransactionPriorityQueue::default(),
            state: HashMap::default(),
            gateway_network,
            batcher_network,
        };

        mempool.txs_queue = TransactionPriorityQueue::from(
            inputs
                .into_iter()
                .map(|input| {
                    // Attempts to insert a key-value pair into the mempool's state. Returns `None`
                    // if the key was not present, otherwise returns the old value while updating
                    // the new value.
                    let prev_value =
                        mempool.state.insert(input.account.address, input.account.state);
                    assert!(
                        prev_value.is_none(),
                        "Sender address: {:?} already exists in the mempool. Can't add {:?} to \
                         the mempool.",
                        input.account.address,
                        input.tx
                    );
                    input.tx
                })
                .collect::<Vec<ThinTransaction>>(),
        );

        mempool
    }

    pub fn empty(
        gateway_network: MempoolNetworkComponent,
        batcher_network: BatcherToMempoolChannels,
    ) -> Self {
        Mempool::new([], gateway_network, batcher_network)
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

    /// Listens asynchronously for network messages and processes them.
    pub async fn run(&mut self) -> Result<()> {
        loop {
            select! {
                optional_gateway_message = self.gateway_network.recv() => {
                    match optional_gateway_message {
                        Some(message) => {
                            self.process_gateway_message(message)?;
                        },
                        // Channel was closed; exit.
                        None => break,
                    }
                }
                optional_batcher_message = self.batcher_network.rx.recv() => {
                    match optional_batcher_message {
                        Some(message) => {
                            self.process_batcher_message(message).await?;
                        },
                        // Channel was closed; exit.
                        None => break,
                    }
                }
            }
        }
        Ok(())
    }

    fn process_gateway_message(&mut self, message: GatewayToMempoolMessage) -> Result<()> {
        match message {
            GatewayToMempoolMessage::AddTransaction(mempool_input) => {
                self.add_tx(mempool_input.tx, mempool_input.account)?;
                Ok(())
            }
        }
    }

    async fn process_batcher_message(&mut self, message: BatcherToMempoolMessage) -> Result<()> {
        match message {
            BatcherToMempoolMessage::GetTransactions(n_txs) => {
                let txs = self.get_txs(n_txs)?;
                self.batcher_network.tx.send(txs).await?;
                Ok(())
            }
        }
    }
}

pub struct MempoolCommunicationWrapper {
    pub mempool: Mutex<Mempool>,
}

#[async_trait]
impl MempoolInterface for MempoolCommunicationWrapper {
    async fn add_tx(&self, mempool_input: MempoolInput) -> MempoolResult<()> {
        self.mempool.lock().await.add_tx(mempool_input.tx, mempool_input.account)
    }

    async fn get_txs(&self, n_txs: usize) -> MempoolResult<Vec<ThinTransaction>> {
        self.mempool.lock().await.get_txs(n_txs)
    }
}

#[async_trait]
impl ComponentMessageExecutor<MempoolInvocationMessages, MempoolInvocationResponses>
    for MempoolCommunicationWrapper
{
    async fn execute(&mut self, message: MempoolInvocationMessages) -> MempoolInvocationResponses {
        match message {
            MempoolInvocationMessages::AddTransaction(mempool_input) => {
                let res = self.add_tx(mempool_input).await;
                MempoolInvocationResponses::AddTransaction(res)
            }
            MempoolInvocationMessages::GetTransactions(n_txs) => {
                let res = self.get_txs(n_txs).await;
                MempoolInvocationResponses::GetTransactions(res)
            }
        }
    }
}
