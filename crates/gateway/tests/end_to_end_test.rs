use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use axum::body::{Body, HttpBody};
use axum::http::{Request, StatusCode};
use hyper::{Client, Response};
use mempool_infra::component_client::ComponentClient;
use mempool_infra::component_server::{ComponentServer, MessageAndResponseSender};
use rstest::rstest;
use starknet_api::transaction::Tip;
use starknet_gateway::config::{
    GatewayNetworkConfig, StatefulTransactionValidatorConfig, StatelessTransactionValidatorConfig,
};
use starknet_gateway::gateway::Gateway;
use starknet_gateway::starknet_api_test_utils::invoke_tx;
use starknet_gateway::state_reader_test_utils::test_state_reader_factory;
use starknet_mempool::mempool::Mempool;
use starknet_mempool_types::mempool_types::{MempoolMessages, MempoolResponses, MempoolTrait};
use tokio::sync::mpsc::channel;
use tokio::task;
use tokio::time::sleep;

async fn set_up_gateway(mempool: Box<dyn MempoolTrait>) -> (IpAddr, u16) {
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let port = 3000;
    let network_config = GatewayNetworkConfig { ip, port };
    let stateless_transaction_validator_config = StatelessTransactionValidatorConfig {
        validate_non_zero_l1_gas_fee: true,
        max_calldata_length: 10,
        max_signature_length: 2,
        ..Default::default()
    };
    let stateful_transaction_validator_config =
        StatefulTransactionValidatorConfig::create_for_testing();
    let state_reader_factory = Arc::new(test_state_reader_factory());

    let gateway = Gateway {
        network_config,
        stateless_transaction_validator_config,
        stateful_transaction_validator_config,
        state_reader_factory,
        mempool,
    };

    // Setup server
    tokio::spawn(async move { gateway.build_server().await });

    // Ensure the server has time to start up
    sleep(Duration::from_millis(1000)).await;
    (ip, port)
}

async fn send_and_verify_transaction(
    ip: IpAddr,
    port: u16,
    tx_json: String,
    expected_response: &str,
) {
    let request = Request::builder()
        .method("POST")
        .uri(format!("http://{}", SocketAddr::from((ip, port))) + "/add_tx")
        .header("content-type", "application/json")
        .body(Body::from(tx_json))
        .unwrap();

    // Create a client
    let client = Client::new();

    // Send a POST request with the transaction data as the body
    let response: Response<Body> = client.request(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let res = response.into_body().collect().await.unwrap().to_bytes();

    assert_eq!(res, expected_response.as_bytes());
}

#[rstest]
#[tokio::test]
async fn test_end_to_end() {
    // Initialize Mempool.
    let (tx_mempool, rx_mempool) =
        channel::<MessageAndResponseSender<MempoolMessages, MempoolResponses>>(32);
    let mempool = Mempool::empty();
    let mut mempool_server = ComponentServer::new(mempool, rx_mempool);
    task::spawn(async move {
        mempool_server.start().await;
    });

    // Initialize Gateway.
    let gateway_mempool_client =
        Box::new(ComponentClient::<MempoolMessages, MempoolResponses>::new(tx_mempool.clone()));
    let (ip, port) = set_up_gateway(gateway_mempool_client).await;

    // Send a transaction.
    let invoke_json = serde_json::to_string(&invoke_tx()).unwrap();
    send_and_verify_transaction(ip, port, invoke_json, "INVOKE").await;

    // Wait for the listener to receive the transactions.
    sleep(Duration::from_secs(2)).await;

    // Check that the mempool received the transaction.
    let mut batcher_mempool_client =
        Box::new(ComponentClient::<MempoolMessages, MempoolResponses>::new(tx_mempool.clone()));
    let mempool_message = batcher_mempool_client.async_get_txs(2).await.unwrap();
    assert_eq!(mempool_message.len(), 1);
    assert_eq!(mempool_message[0].tip, Tip(0));
}
