use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use axum::body::{Body, Bytes, HttpBody};
use axum::http::{Request, StatusCode};
use mempool_infra::component_server::ComponentServer;
use pretty_assertions::assert_str_eq;
use rstest::rstest;
use starknet_api::external_transaction::ExternalTransaction;
use starknet_gateway::config::{
    GatewayConfig, GatewayNetworkConfig, StatefulTransactionValidatorConfig,
    StatelessTransactionValidatorConfig,
};
use starknet_gateway::gateway::Gateway;
use starknet_gateway::starknet_api_test_utils::{external_invoke_tx_to_json, invoke_tx};
use starknet_gateway::state_reader_test_utils::test_state_reader_factory;
use starknet_mempool::mempool::{Mempool, MempoolCommunicationWrapper};
use starknet_mempool_types::mempool_types::{
    BatcherToMempoolChannels, BatcherToMempoolMessage, GatewayNetworkComponent,
    GatewayToMempoolMessage, MempoolClient, MempoolMessageAndResponseSender,
    MempoolNetworkComponent, MempoolToBatcherMessage, MempoolToGatewayMessage,
};
use tokio::sync::mpsc::channel;
use tokio::sync::Mutex;
use tokio::task;
use tower::ServiceExt;

// TODO(Ayelet): add test cases for declare and deploy account transactions.
#[rstest]
#[case::invoke(invoke_tx(), "INVOKE")]
#[tokio::test]
async fn test_routes(
    #[case] external_invoke_tx: ExternalTransaction,
    #[case] expected_response: &str,
) {
    let tx_json = external_invoke_tx_to_json(&external_invoke_tx);
    let request = Request::post("/add_tx")
        .header("content-type", "application/json")
        .body(Body::from(tx_json))
        .unwrap();

    let response = check_request(request, StatusCode::OK).await;

    assert_str_eq!(expected_response, String::from_utf8_lossy(&response));
}

#[rstest]
#[tokio::test]
#[should_panic]
// FIXME: Currently is_alive is not implemented, fix this once it is implemented.
async fn test_is_alive() {
    let request = Request::get("/is_alive").body(Body::empty()).unwrap();
    // Status code doesn't matter, this panics ATM.
    check_request(request, StatusCode::default()).await;
}

async fn check_request(request: Request<Body>, status_code: StatusCode) -> Bytes {
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let port = 3000;
    let network_config: GatewayNetworkConfig = GatewayNetworkConfig { ip, port };

    let stateless_transaction_validator_config = StatelessTransactionValidatorConfig {
        max_calldata_length: 1000,
        max_signature_length: 2,
        ..Default::default()
    };
    let stateful_transaction_validator_config =
        StatefulTransactionValidatorConfig::create_for_testing();

    let config = GatewayConfig {
        network_config,
        stateless_transaction_validator_config,
        stateful_transaction_validator_config,
    };

    // TODO: remove NetworkComponent, GatewayToMempoolMessage, and MempoolToGatewayMessage.
    let (tx_gateway_to_mempool, rx_gateway_to_mempool) = channel::<GatewayToMempoolMessage>(1);
    let (tx_mempool_to_gateway, rx_mempool_to_gateway) = channel::<MempoolToGatewayMessage>(1);
    let network_component =
        GatewayNetworkComponent::new(tx_gateway_to_mempool, rx_mempool_to_gateway);
    let state_reader_factory = Arc::new(test_state_reader_factory());

    // Initialize a Mempool.
    let mempool_to_gateway_network =
        MempoolNetworkComponent::new(tx_mempool_to_gateway, rx_gateway_to_mempool);

    let (_tx_batcher_to_mempool, rx_batcher_to_mempool) = channel::<BatcherToMempoolMessage>(1);
    let (tx_mempool_to_batcher, _rx_mempool_to_batcher) = channel::<MempoolToBatcherMessage>(1);

    let batcher_channels =
        BatcherToMempoolChannels { rx: rx_batcher_to_mempool, tx: tx_mempool_to_batcher };

    let (tx_mempool, rx_mempool) = channel::<MempoolMessageAndResponseSender>(32);

    // Initialize Gateway.
    let mempool_client = Box::new(MempoolClient::new(tx_mempool.clone()));

    let mempool = Mempool::empty(mempool_to_gateway_network, batcher_channels);
    // TODO(Tsabary, 1/6/2024): Wrap with a dedicated create_mempool_server function.
    let mut mempool_server = ComponentServer::new(
        MempoolCommunicationWrapper { mempool: Mutex::new(mempool) },
        rx_mempool,
    );
    task::spawn(async move {
        mempool_server.start().await;
    });

    // TODO: Add fixture.
    let gateway = Gateway::new(config, network_component, state_reader_factory, mempool_client);

    let response = gateway.app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), status_code);

    response.into_body().collect().await.unwrap().to_bytes()
}
