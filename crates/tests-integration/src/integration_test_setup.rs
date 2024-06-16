use std::net::SocketAddr;
use std::sync::Arc;

use itertools::zip_eq;
use starknet_api::rpc_transaction::RPCTransaction;
use starknet_api::transaction::TransactionHash;
use starknet_gateway::config::GatewayNetworkConfig;
use starknet_gateway::errors::GatewayError;
use starknet_gateway::starknet_api_test_utils::MultiAccountTransactionGenerator;
use starknet_mempool::communication::create_mempool_server;
use starknet_mempool::mempool::Mempool;
use starknet_mempool_types::communication::{
    MempoolClient, MempoolClientImpl, MempoolRequestAndResponseSender,
};
use starknet_mempool_types::mempool_types::ThinTransaction;
use starknet_task_executor::executor::TaskExecutor;
use starknet_task_executor::tokio_executor::TokioExecutor;
use tokio::runtime::Handle;
use tokio::sync::mpsc::channel;
use tokio::task::JoinHandle;

use crate::integration_test_utils::{create_gateway, GatewayClient};

pub struct IntegrationTestSetup {
    pub task_executor: TokioExecutor,
    pub gateway_client: GatewayClient,
    // TODO(MockBatcher).
    pub batcher_mempool_client: MempoolClientImpl,

    pub gateway_handle: JoinHandle<()>,
    pub mempool_handle: JoinHandle<()>,
}

impl IntegrationTestSetup {
    pub async fn new_with_tx_generator(
        n_accounts: usize,
    ) -> (Self, MultiAccountTransactionGenerator) {
        let integration_test_setup = Self::new(n_accounts).await;
        let tx_generator = MultiAccountTransactionGenerator::new(n_accounts);

        (integration_test_setup, tx_generator)
    }

    pub async fn new(n_accounts: usize) -> Self {
        let handle = Handle::current();
        let task_executor = TokioExecutor::new(handle);

        // TODO(Tsabary): wrap creation of channels in dedicated functions, take channel capacity
        // from config.
        const MEMPOOL_INVOCATIONS_QUEUE_SIZE: usize = 32;
        let (tx_mempool, rx_mempool) =
            channel::<MempoolRequestAndResponseSender>(MEMPOOL_INVOCATIONS_QUEUE_SIZE);
        // Build and run gateway; initialize a gateway client.
        let gateway_mempool_client = MempoolClientImpl::new(tx_mempool.clone());
        let gateway = create_gateway(Arc::new(gateway_mempool_client), n_accounts).await;
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
        // TODO(MockBatcher)
        let batcher_mempool_client = MempoolClientImpl::new(tx_mempool.clone());

        // Build and run mempool.
        let mut mempool_server = create_mempool_server(Mempool::empty(), rx_mempool);
        let mempool_handle = task_executor.spawn_with_handle(async move {
            mempool_server.start().await;
        });

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

    pub async fn assert_get_txs_eq(
        &mut self,
        n_txs: usize,
        expected_tx_hashes: &[TransactionHash],
    ) {
        let mempool_txs = self.get_txs(n_txs).await;

        assert!(
            zip_eq(expected_tx_hashes, mempool_txs)
                // Deref the inner mempool tx type.
                .all(|(&expected_tx_hash, mempool_tx)| expected_tx_hash == mempool_tx.tx_hash)
        );
    }
}
