use starknet_mempool_infra::component_definitions::ComponentRequestAndResponseSender;
use starknet_mempool_types::communication::{
    MempoolClient, MempoolClientImpl, MempoolRequest, MempoolResponse,
};
use starknet_mempool_types::mempool_types::ThinTransaction;
use tokio::sync::mpsc::{Sender, UnboundedReceiver};

use crate::integration_test_utils::BatcherCommand;

pub struct MockBatcher {
    rx_commands: UnboundedReceiver<BatcherCommand>,
    mempool_client: MempoolClientImpl,
}

impl MockBatcher {
    pub fn new(
        rx_commands: UnboundedReceiver<BatcherCommand>,
        mempool_sender: Sender<ComponentRequestAndResponseSender<MempoolRequest, MempoolResponse>>,
    ) -> Self {
        Self { rx_commands, mempool_client: MempoolClientImpl::new(mempool_sender) }
    }

    async fn get_txs(&self, n_txs: usize) -> Vec<ThinTransaction> {
        self.mempool_client.get_txs(n_txs).await.unwrap()
    }

    pub async fn run(&mut self) {
        while let Some(message) = self.rx_commands.recv().await {
            match message {
                BatcherCommand::GetTxs(n_txs, tx_response) => {
                    let txs = self.get_txs(n_txs).await;
                    tx_response.send(txs).unwrap();
                }
            }
        }
    }
}
