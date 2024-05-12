use std::collections::HashMap;

use mempool_infra::network_component::CommunicationInterface;
use starknet_api::core::ContractAddress;
use starknet_mempool_types::mempool_types::{
    AccountState, BatcherMempoolNetworkComponent, BatcherToMempoolMessage, MempoolToBatcherMessage,
};
use tokio::sync::mpsc::channel;

pub struct MockBatcher {
    pub state: HashMap<ContractAddress, AccountState>,
    pub network: BatcherMempoolNetworkComponent,
}

impl MockBatcher {
    pub fn new(state: HashMap<ContractAddress, AccountState>) -> Self {
        let (_, rx_mempool_to_batcher) = channel::<MempoolToBatcherMessage>(1);
        let (tx_batcher_to_mempool, _) = channel::<BatcherToMempoolMessage>(1);

        MockBatcher {
            state,
            network: BatcherMempoolNetworkComponent::new(
                tx_batcher_to_mempool,
                rx_mempool_to_batcher,
            ),
        }
    }

    pub async fn retrieve_txs(&mut self, n_txs: u8) -> Result<(), Box<dyn std::error::Error>> {
        // Send a message to the mempool asking for a number of transactions
        self.network.send(BatcherToMempoolMessage::GetTxs(n_txs)).await?;

        // Receive the transactions from the mempool
        if let Some(txs) = self.network.recv().await {
            for tx in txs {
                self.state
                    .entry(tx.contract_address)
                    .and_modify(|account_state| account_state.update_state().unwrap());
            }
        }

        Ok(())
    }
}
