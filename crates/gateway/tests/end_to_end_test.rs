use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use rstest::rstest;
use starknet_api::transaction::Tip;
use starknet_gateway::config::{
    GatewayConfig, StatefulTransactionValidatorConfig, StatelessTransactionValidatorConfig,
};
use starknet_gateway::gateway::Gateway;
use starknet_gateway::gateway_client;
use starknet_gateway::starknet_api_test_utils::invoke_tx;
use starknet_gateway::state_reader_test_utils::test_state_reader_factory;
use starknet_mempool::mempool::{create_mempool_server, Mempool};
use starknet_mempool_types::mempool_types::{
    GatewayNetworkComponent, GatewayToMempoolMessage, MempoolClient, MempoolClientImpl, MempoolInput, MempoolNetworkComponent, MempoolRequestAndResponseSender, MempoolToGatewayMessage
    MempoolClient, MempoolClientInterface, MempoolMessageAndResponseSender,
};
use tokio::sync::mpsc::channel;
use tokio::task;
use tokio::time::sleep;

const MEMPOOL_INVOCATIONS_QUEUE_SIZE: usize = 32;

#[rstest]
#[tokio::test]
async fn test_end_to_end() {
    // Initialize Mempool.
    // TODO(Tsabary): wrap creation of channels in dedicated functions, take channel capacity from
    // config.
    let (tx_mempool, rx_mempool) =
        channel::<MempoolRequestAndResponseSender>(MEMPOOL_INVOCATIONS_QUEUE_SIZE);
    let mempool = Mempool::empty();
    let mut mempool_server = create_mempool_server(mempool, rx_mempool);

    task::spawn(async move {
        mempool_server.start().await;
    });

    // Initialize Gateway.
    let gateway_mempool_client = Box::new(MempoolClientImpl::new(tx_mempool.clone()));
    let socket_addr = set_up_gateway(gateway_mempool_client).await;

    // Send a transaction.
    let external_tx = invoke_tx();
    let gateway_client = gateway_client::GatewayClient::new(socket_addr);
    gateway_client.assert_add_tx_success(&external_tx).await;

    let batcher_mempool_client = Box::new(MempoolClientImpl::new(tx_mempool.clone()));
    let mempool_message =
        batcher_mempool_client.get_txs(2).await.expect("Communication should succeed").unwrap();

    assert_eq!(mempool_message.len(), 1);
    assert_eq!(mempool_message[0].tip, Tip(0));
}
