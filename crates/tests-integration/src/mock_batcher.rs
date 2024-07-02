use starknet_mempool_infra::component_definitions::ComponentRequestAndResponseSender;
use starknet_mempool_types::communication::{
    MempoolClient, MempoolClientImpl, MempoolRequest, MempoolResponse,
};
use starknet_mempool_types::mempool_types::ThinTransaction;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::integration_test_utils::BatcherCommand;

pub struct MockBatcher {
    trigger_receiver: Receiver<BatcherCommand>,
    mempool_client: MempoolClientImpl,
}

impl MockBatcher {
    pub fn new(
        trigger_receiver: Receiver<BatcherCommand>,
        mempool_sender: Sender<ComponentRequestAndResponseSender<MempoolRequest, MempoolResponse>>,
    ) -> Self {
        Self { trigger_receiver, mempool_client: MempoolClientImpl::new(mempool_sender) }
    }

    async fn get_txs(&self, n_txs: usize) -> Vec<ThinTransaction> {
        self.mempool_client.get_txs(n_txs).await.unwrap()
    }

    pub async fn run(&mut self) {
        while let Some(message) = self.trigger_receiver.recv().await {
            match message {
                BatcherCommand::GetTxs(n_txs, sender) => {
                    let txs = self.get_txs(n_txs).await;
                    sender.send(txs).unwrap();
                }
            }
        }
    }
}
