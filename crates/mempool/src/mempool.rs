use std::collections::HashMap;

use async_trait::async_trait;
use starknet_api::core::ContractAddress;
use starknet_api::transaction::TransactionHash;
use starknet_mempool_infra::component_definitions::ComponentRequestHandler;
use starknet_mempool_infra::component_server::ComponentServer;
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::{
    AccountState, MempoolInput, MempoolRequest, MempoolRequestAndResponseSender, MempoolResponse,
    MempoolResult, ThinTransaction,
};
use tokio::sync::mpsc::Receiver;

use crate::priority_queue::{AddressStore, TransactionPriorityQueue};

#[cfg(test)]
#[path = "mempool_test.rs"]
pub mod mempool_test;

#[derive(Default)]
pub struct Mempool {
    // TODO: add docstring explaining visibility and coupling of the fields.
    txs_queue: TransactionPriorityQueue,
    address_to_store: HashMap<ContractAddress, AddressStore>,
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
    // back. TODO: Consider renaming to `pop_txs` to be more consistent with the standard library.
    // TODO: If `n_txs` is greater than the number of transactions in `txs_queue`, it will also
    // check and add transactions from `address_to_store`.
    pub fn get_txs(&mut self, n_txs: usize) -> MempoolResult<Vec<ThinTransaction>> {
        let txs = self.txs_queue.pop_last_chunk(n_txs);
        for tx in &txs {
            if let Some(address_queue) = self.address_to_store.get_mut(&tx.sender_address) {
                address_queue.pop_front();

                if address_queue.is_empty() {
                    self.address_to_store.remove(&tx.sender_address);
                } else if let Some(next_tx) = address_queue.top() {
                    self.txs_queue.push(next_tx.clone());
                }
            }
        }

        Ok(txs)
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
        let address_queue = self.address_to_store.entry(tx.sender_address).or_default();

        if address_queue.contains(&tx) {
            return Err(MempoolError::DuplicateTransaction { tx_hash: tx.tx_hash });
        }

        address_queue.push(tx.clone());
        if address_queue.len() == 1 {
            self.txs_queue.push(tx);
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
        self.mempool.add_tx(mempool_input.tx)
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
