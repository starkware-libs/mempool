use std::net::SocketAddr;
use std::sync::Arc;

use starknet_api::rpc_transaction::RPCTransaction;
use starknet_api::transaction::TransactionHash;
use starknet_gateway::config::GatewayNetworkConfig;
use starknet_gateway::errors::GatewayError;
use starknet_mempool_node::communication::create_node_channels;
use starknet_mempool_node::com_clients::create_node_clients;
use starknet_mempool_node::com_servers::{create_servers, get_server_future};
use starknet_mempool_node::components::create_components;
use starknet_mempool_types::communication::{MempoolClient, MempoolClientImpl};
use starknet_mempool_types::mempool_types::ThinTransaction;
use starknet_task_executor::executor::TaskExecutor;
use starknet_task_executor::tokio_executor::TokioExecutor;
use tokio::runtime::Handle;
use tokio::task::JoinHandle;

use crate::integration_test_utils::{adjust_gateway_state_reader, create_config, GatewayClient};

pub struct IntegrationTestSetup {
    pub task_executor: TokioExecutor,
    pub gateway_client: GatewayClient,
    // TODO(MockBatcher).
    pub batcher_mempool_client: Arc<MempoolClientImpl>,
    pub gateway_handle: JoinHandle<()>,
    pub mempool_handle: JoinHandle<()>,
}

impl IntegrationTestSetup {
    pub async fn new() -> Self {
        let handle = Handle::current();
        let task_executor = TokioExecutor::new(handle);

        // Build and run gateway; initialize a gateway client.
        let config = create_config().await;

        let comm_channels = create_node_channels();

        let com_clients = create_node_clients(&config, &comm_channels);

        let mut components = create_components(&config, &com_clients);
        adjust_gateway_state_reader(&mut components).await;

        let com_servers = create_servers(&config, comm_channels, components);

        let GatewayNetworkConfig { ip, port } = config.gateway_config.network_config;
        let gateway_client = GatewayClient::new(SocketAddr::from((ip, port)));

        let gateway_future = get_server_future("Gateway", true, com_servers.gateway);
        let gateway_handle = task_executor.spawn_with_handle(gateway_future);

        // Wait for server to spin up.
        // TODO(Gilad): Replace with a persistant Client with a built-in retry mechanism,
        // to avoid the sleep and to protect against CI flakiness.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Build Batcher.
        // TODO(MockBatcher)
        // let batcher_mempool_client =
        // MempoolClientImpl::new(comm_channels.mempool_channel.tx.clone());
        let batcher_mempool_client = com_clients.mempool_client.unwrap().clone();

        // Build and run mempool.
        let mempool_future = get_server_future("Mempool", true, com_servers.mempool);
        let mempool_handle = task_executor.spawn_with_handle(mempool_future);

        Self {
            task_executor,
            gateway_client,
            batcher_mempool_client,
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

    pub async fn get_txs(&mut self, n_txs: usize) -> Vec<ThinTransaction> {
        let batcher_mempool_client = self.batcher_mempool_client.clone();
        self.task_executor
            .spawn(async move { batcher_mempool_client.get_txs(n_txs).await.unwrap() })
            .await
            .unwrap()
    }
}
