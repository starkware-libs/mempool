use std::net::SocketAddr;
use std::sync::Arc;

use starknet_api::rpc_transaction::RPCTransaction;
use starknet_api::transaction::TransactionHash;
use starknet_gateway::config::GatewayNetworkConfig;
use starknet_gateway::errors::GatewayError;
use starknet_mempool::communication::create_mempool_server;
use starknet_mempool::mempool::Mempool;
use starknet_mempool_infra::component_server::ComponentServerStarter;
use starknet_mempool_types::communication::{MempoolClientImpl, MempoolRequestAndResponseSender};
use starknet_mempool_types::mempool_types::ThinTransaction;
use starknet_task_executor::tokio_executor::TokioExecutor;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use crate::integration_test_utils::{create_gateway, BatcherCommand, GatewayClient};
use crate::mock_batcher::MockBatcher;

pub struct IntegrationTestSetup {
    pub task_executor: TokioExecutor,
    pub gateway_client: GatewayClient,

    pub batcher_handle: JoinHandle<()>,
    tx_batcher: Sender<BatcherCommand>,

    pub gateway_handle: JoinHandle<()>,
    pub mempool_handle: JoinHandle<()>,
}

impl IntegrationTestSetup {
    pub async fn new(n_initialized_account_contracts: u16) -> Self {
        let handle = Handle::current();
        let task_executor = TokioExecutor::new(handle);

        // TODO(Tsabary): wrap creation of channels in dedicated functions, take channel capacity
        // from config.
        const MESSAGE_QUEUE_SIZE: usize = 32;
        let (tx_mempool, rx_mempool) =
            channel::<MempoolRequestAndResponseSender>(MESSAGE_QUEUE_SIZE);
        // Build and run gateway; initialize a gateway client.
        let gateway_mempool_client = MempoolClientImpl::new(tx_mempool.clone());
        let gateway =
            create_gateway(Arc::new(gateway_mempool_client), n_initialized_account_contracts).await;
        let GatewayNetworkConfig { ip, port } = gateway.config.network_config;
        let gateway_client = GatewayClient::new(SocketAddr::from((ip, port)));
        let gateway_handle = task_executor.spawn_with_handle(async move {
            gateway.run().await.unwrap();
        });

        // Wait for server to spin up.
        // TODO(Gilad): Replace with a persistant Client with a built-in retry mechanism,
        // to avoid the sleep and to protect against CI flakiness.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Build Batcher.
        let (tx_batcher, rx_batcher) = channel::<BatcherCommand>(MESSAGE_QUEUE_SIZE);
        let mut batcher = MockBatcher::new(rx_batcher, tx_mempool);
        let batcher_handle = task_executor.spawn_with_handle(async move {
            batcher.run().await;
        });

        // Build and run mempool.
        let mut mempool_server = create_mempool_server(Mempool::empty(), rx_mempool);
        let mempool_handle = task_executor.spawn_with_handle(async move {
            mempool_server.start().await;
        });

        Self {
            task_executor,
            gateway_client,
            batcher_handle,
            tx_batcher,
            gateway_handle,
            mempool_handle,
        }
    }

    pub async fn assert_add_tx_success(&self, tx: &RPCTransaction) -> TransactionHash {
        self.gateway_client.assert_add_tx_success(tx).await
    }

    pub async fn assert_add_tx_error(&self, tx: &RPCTransaction) -> GatewayError {
        self.gateway_client.assert_add_tx_error(tx).await
    }

    pub async fn get_txs(&self, n_txs: usize) -> Vec<ThinTransaction> {
        let (response_tx, response_rx) = oneshot::channel::<Vec<ThinTransaction>>();
        self.tx_batcher.send(BatcherCommand::GetTxs(n_txs, response_tx)).await.unwrap();
        response_rx.await.unwrap()
    }
}
