use std::collections::HashMap;

use async_trait::async_trait;
use starknet_api::core::ContractAddress;
use starknet_api::transaction::TransactionHash;
use starknet_mempool_infra::component_server::{ComponentRequestHandler, ComponentServer};
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::{
    Account, AccountState, MempoolInput, MempoolRequest, MempoolRequestAndResponseSender,
    MempoolResponse, MempoolResult, ThinTransaction,
};
use tokio::sync::mpsc::Receiver;

use crate::priority_queue::{AddressPriorityQueue, TransactionPriorityQueue};

#[cfg(test)]
#[path = "mempool_test.rs"]
pub mod mempool_test;

#[derive(Debug)]
pub struct Mempool {
    // TODO: add docstring explaining visibility and coupling of the fields.
    txs_queue: TransactionPriorityQueue,
    address_to_queue: HashMap<ContractAddress, AddressPriorityQueue>,
    state: HashMap<ContractAddress, AccountState>,
}

impl Mempool {
    pub fn new(inputs: impl IntoIterator<Item = MempoolInput>) -> MempoolResult<Self> {
        let mut mempool = Mempool {
            txs_queue: TransactionPriorityQueue::default(),
            address_to_queue: HashMap::default(),
            state: HashMap::default(),
        };

        for input in inputs {
            mempool.handle_tx(input.tx, input.account)?;
        }

        Ok(mempool)
    }

    pub fn empty() -> MempoolResult<Self> {
        Mempool::new([])
    }

    /// Retrieves up to `n_txs` transactions with the highest priority from the mempool.
    /// Transactions are guaranteed to be unique across calls until `commit_block` is invoked.
    // TODO: the last part about commit_block is incorrect if we delete txs in get_txs and then push
    // back. TODO: Consider renaming to `pop_txs` to be more consistent with the standard library.
    pub fn get_txs(&mut self, n_txs: usize) -> MempoolResult<Vec<ThinTransaction>> {
        let txs = self.txs_queue.pop_last_chunk(n_txs);
        for tx in &txs {
            if let Some(address_queue) = self.address_to_queue.get_mut(&tx.sender_address) {
                address_queue.pop_front();
                self.state
                    .insert(tx.sender_address, AccountState { nonce: tx.nonce.try_increment()? });

                if address_queue.is_empty() {
                    self.address_to_queue.remove(&tx.sender_address);
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
        self.handle_tx(tx, account)?;
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

    fn handle_tx(&mut self, tx: ThinTransaction, account: Account) -> MempoolResult<()> {
        let address_queue = self
            .address_to_queue
            .entry(tx.sender_address)
            .or_insert_with(|| AddressPriorityQueue(Vec::new()));

        if address_queue.contains(&tx) {
            return Err(MempoolError::DuplicateTransaction { tx_hash: tx.tx_hash });
        }

        address_queue.push(tx.clone());
        self.state.insert(account.sender_address, account.state);

        if let Some(state) = self.state.get(&tx.sender_address) {
            if state.nonce == tx.nonce {
                self.txs_queue.push(tx);
            }
        }

        Ok(())
    }
}

/// Wraps the mempool to enable inbound async communication from other components.
pub struct MempoolCommunicationWrapper {
    mempool: Mempool,
}

impl MempoolCommunicationWrapper {
    pub fn new(mempool: Mempool) -> Self {
        MempoolCommunicationWrapper { mempool }
    }

    fn add_tx(&mut self, mempool_input: MempoolInput) -> MempoolResult<()> {
        self.mempool.add_tx(mempool_input.tx, mempool_input.account)
    }

    fn get_txs(&mut self, n_txs: usize) -> MempoolResult<Vec<ThinTransaction>> {
        self.mempool.get_txs(n_txs)
    }
}

#[async_trait]
impl ComponentRequestHandler<MempoolRequest, MempoolResponse> for MempoolCommunicationWrapper {
    async fn handle_request(&mut self, request: MempoolRequest) -> MempoolResponse {
        match request {
            MempoolRequest::AddTransaction(mempool_input) => {
                MempoolResponse::AddTransaction(self.add_tx(mempool_input))
            }
            MempoolRequest::GetTransactions(n_txs) => {
                MempoolResponse::GetTransactions(self.get_txs(n_txs))
            }
        }
    }
}

type MempoolCommunicationServer =
    ComponentServer<MempoolCommunicationWrapper, MempoolRequest, MempoolResponse>;

pub fn create_mempool_server(
    mempool: Mempool,
    rx_mempool: Receiver<MempoolRequestAndResponseSender>,
) -> MempoolCommunicationServer {
    let mempool_communication_wrapper = MempoolCommunicationWrapper::new(mempool);
    ComponentServer::new(mempool_communication_wrapper, rx_mempool)
}
