use std::net::SocketAddr;

use starknet_api::external_transaction::ExternalTransaction;
use starknet_gateway::config::GatewayNetworkConfig;
use starknet_gateway::gateway::Gateway;
use starknet_mempool::mempool::Mempool;
use starknet_mempool_types::mempool_types::{
    BatcherToMempoolChannels, BatcherToMempoolMessage, GatewayNetworkComponent,
    GatewayToMempoolMessage, MempoolNetworkComponent, MempoolToBatcherMessage,
    MempoolToGatewayMessage, ThinTransaction,
};
use starknet_task_executor::executor::TaskExecutor;
use starknet_task_executor::tokio_executor::TokioExecutor;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::integration_test_utils::GatewayClient;

pub struct IntegrationTestSetup {
    pub executor: TokioExecutor,
    pub gateway_client: GatewayClient,
    // TODO(MockBatcher).
    pub tx_batcher_to_mempool: Sender<BatcherToMempoolMessage>,
    pub rx_mempool_to_batcher: Mutex<Receiver<Vec<ThinTransaction>>>,

    pub gateway_handle: JoinHandle<()>,
    pub mempool_handle: JoinHandle<()>,
}

impl IntegrationTestSetup {
    pub async fn new() -> Self {
        let handle = Handle::current();
        let executor = TokioExecutor::new(handle);

        let (gateway_to_mempool_network, mempool_to_gateway_network) =
            initialize_gateway_network_channels();

        // Build and run Gateway and initialize a gateway client.
        let gateway = Gateway::create_for_testing(gateway_to_mempool_network).await;
        let GatewayNetworkConfig { ip, port } = gateway.config.network_config;
        let gateway_client = GatewayClient::new(SocketAddr::from((ip, port)));
        let gateway_handle = executor.spawn_with_handle(async move {
            gateway.run().await.unwrap();
        });

        // Wait for server to spin up.
        // TODO(Gilad): Replace with a persistant Client with a built-in retry mechanism,
        // to avoid the sleep and to protect against CI flakiness.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Build Batcher.
        // TODO(MockBatcher)
        let (tx_batcher_to_mempool, rx_batcher_to_mempool) = channel::<BatcherToMempoolMessage>(1);
        let (tx_mempool_to_batcher, rx_mempool_to_batcher) = channel::<MempoolToBatcherMessage>(1);
        let batcher_channels =
            BatcherToMempoolChannels { rx: rx_batcher_to_mempool, tx: tx_mempool_to_batcher };
        // Build and run mempool.
        let mut mempool = Mempool::empty(mempool_to_gateway_network, batcher_channels);
        let mempool_handle = executor.spawn_with_handle(async move {
            mempool.run().await.unwrap();
        });

        Self {
            executor,
            gateway_client,
            tx_batcher_to_mempool,
            rx_mempool_to_batcher: Mutex::new(rx_mempool_to_batcher),
            gateway_handle,
            mempool_handle,
        }
    }

    pub async fn assert_add_tx_success(&self, tx: &ExternalTransaction, expected: &str) {
        self.gateway_client.assert_add_tx_success(tx, expected).await;
    }

    pub async fn get_txs(&self, n_txs: usize) -> Vec<ThinTransaction> {
        let batcher_to_mempool_message = BatcherToMempoolMessage::GetTransactions(n_txs);
        let tx_batcher_to_mempool = self.tx_batcher_to_mempool.clone();
        self.executor
            .spawn(async move {
                tx_batcher_to_mempool.send(batcher_to_mempool_message).await.unwrap();
            })
            .await
            .unwrap();

        self.rx_mempool_to_batcher.lock().await.recv().await.unwrap()
    }
}

fn initialize_gateway_network_channels() -> (GatewayNetworkComponent, MempoolNetworkComponent) {
    let (tx_gateway_to_mempool, rx_gateway_to_mempool) = channel::<GatewayToMempoolMessage>(1);
    let (tx_mempool_to_gateway, rx_mempool_to_gateway) = channel::<MempoolToGatewayMessage>(1);

    (
        GatewayNetworkComponent::new(tx_gateway_to_mempool, rx_mempool_to_gateway),
        MempoolNetworkComponent::new(tx_mempool_to_gateway, rx_gateway_to_mempool),
    )
}
