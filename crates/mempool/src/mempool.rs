use std::collections::HashMap;

use anyhow::Result;
use mempool_infra::network_component::CommunicationInterface;
use starknet_api::core::ContractAddress;
use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::{
    Account, AccountState, BatcherToMempoolChannels, BatcherToMempoolMessage,
    GatewayToMempoolMessage, MempoolInput, MempoolNetworkComponent, MempoolResult, ThinTransaction,
};
use tokio::select;

use crate::priority_queue::{
    AddressPriorityQueue, PrioritizedTransaction, PriorityQueueTxResult, TransactionPriorityQueue,
};

#[cfg(test)]
#[path = "mempool_test.rs"]
pub mod mempool_test;

pub struct Mempool {
    // TODO: add docstring explaining visibility and coupling of the fields.
    pub gateway_network: MempoolNetworkComponent,
    batcher_network: BatcherToMempoolChannels,
    txs_queue: TransactionPriorityQueue,
    addresses_to_queues: HashMap<ContractAddress, AddressPriorityQueue>,
    state: HashMap<ContractAddress, AccountState>,
}

impl Mempool {
    pub fn new(
        inputs: impl IntoIterator<Item = MempoolInput>,
        gateway_network: MempoolNetworkComponent,
        batcher_network: BatcherToMempoolChannels,
    ) -> MempoolResult<Self> {
        let mut mempool = Mempool {
            txs_queue: TransactionPriorityQueue::default(),
            addresses_to_queues: HashMap::default(),
            state: HashMap::default(),
            gateway_network,
            batcher_network,
        };

        for input in inputs {
            let address_queue = mempool
                .addresses_to_queues
                .entry(input.account.address)
                .or_insert_with(|| AddressPriorityQueue(Vec::new()));

            let pq_result = address_queue.handle_tx(input.tx.clone());

            match pq_result {
                PriorityQueueTxResult::Duplicate => {
                    return Err(MempoolError::DuplicateTransaction { tx_hash: input.tx.tx_hash });
                }
                PriorityQueueTxResult::Replace(old_tx) => {
                    mempool.txs_queue.remove(&PrioritizedTransaction(old_tx));
                    mempool.txs_queue.push(input.tx);
                }
                PriorityQueueTxResult::New => {
                    mempool.txs_queue.push(input.tx);
                }
                PriorityQueueTxResult::Ignore => {}
            }

            mempool.state.insert(input.account.address, input.account.state);
        }

        Ok(mempool)
    }

    pub fn empty(
        gateway_network: MempoolNetworkComponent,
        batcher_network: BatcherToMempoolChannels,
    ) -> MempoolResult<Self> {
        Mempool::new([], gateway_network, batcher_network)
    }

    /// Retrieves up to `n_txs` transactions with the highest priority from the mempool.
    /// Transactions are guaranteed to be unique across calls until `commit_block` is invoked.
    // TODO: the last part about commit_block is incorrect if we delete txs in get_txs and then push
    // back. TODO: Consider renaming to `pop_txs` to be more consistent with the standard library.
    pub fn get_txs(&mut self, n_txs: usize) -> MempoolResult<Vec<ThinTransaction>> {
        let txs = self.txs_queue.pop_last_chunk(n_txs);
        for tx in &txs {
            if let Some(address_queue) = self.addresses_to_queues.get_mut(&tx.sender_address) {
                address_queue.pop();
                self.state
                    .insert(tx.sender_address, AccountState { nonce: tx.nonce.try_increment()? });

                if address_queue.is_empty() {
                    self.addresses_to_queues.remove(&tx.sender_address);
                } else if let Some(next_tx) = address_queue.top() {
                    if let Some(sender_state) = self.state.get(&tx.sender_address) {
                        if sender_state.nonce == next_tx.nonce {
                            self.txs_queue.push(next_tx.clone());
                        }
                    }
                }
            }
        }

        Ok(txs)
    }

    /// Adds a new transaction to the mempool.
    /// TODO: support fee escalation and transactions with future nonces.
    /// TODO: change input type to `MempoolInput`.
    pub fn add_tx(&mut self, tx: ThinTransaction, account: Account) -> MempoolResult<()> {
        let address_queue = self
            .addresses_to_queues
            .entry(tx.sender_address)
            .or_insert_with(|| AddressPriorityQueue(Vec::new()));

        if address_queue.0.contains(&tx) {
            return Err(MempoolError::DuplicateTransaction { tx_hash: tx.tx_hash });
        }

        self.state.insert(tx.sender_address, account.state);
        address_queue.0.push(tx.clone());

        if let Some(state) = self.state.get(&tx.sender_address) {
            if state.nonce == tx.nonce {
                self.txs_queue.push(tx);
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
