use async_trait::async_trait;
use starknet_mempool_infra::component_definitions::ComponentRequestHandler;
use starknet_mempool_infra::component_server::ComponentServer;
use starknet_mempool_types::communication::{
    MempoolRequest, MempoolRequestAndResponseSender, MempoolResponse,
};
use starknet_mempool_types::mempool_types::{MempoolInput, MempoolResult, ThinTransaction};
use tokio::sync::mpsc::Receiver;

use crate::mempool::Mempool;

pub fn create_mempool_server(
    mempool: Mempool,
    rx_mempool: Receiver<MempoolRequestAndResponseSender>,
) -> ComponentServer<MempoolCommunicationWrapper, MempoolRequest, MempoolResponse> {
    let communication_wrapper = MempoolCommunicationWrapper::new(mempool);
    ComponentServer::new(communication_wrapper, rx_mempool)
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
        self.mempool.add_tx(mempool_input)
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
