use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use axum::body::{Body, Bytes, HttpBody};
use axum::http::{Request, StatusCode};
use pretty_assertions::assert_str_eq;
use rstest::rstest;
use starknet_api::external_transaction::ExternalTransaction;
use starknet_gateway::config::{
    GatewayConfig, GatewayNetworkConfig, StatefulTransactionValidatorConfig,
    StatelessTransactionValidatorConfig,
};
use starknet_gateway::gateway::Gateway;
use starknet_gateway::starknet_api_test_utils::{external_tx_to_json, invoke_tx};
use starknet_gateway::state_reader_test_utils::test_state_reader_factory;
use starknet_mempool_types::mempool_types::{
    GatewayNetworkComponent, GatewayToMempoolMessage, MempoolToGatewayMessage,
};
use tokio::sync::mpsc::channel;
use tower::ServiceExt;

// TODO(Ayelet): add test cases for declare transaction.
#[rstest]
#[case::invoke(invoke_tx(), "INVOKE")]
#[tokio::test]
async fn test_routes(
    #[case] external_invoke_tx: ExternalTransaction,
    #[case] expected_response: &str,
) {
    let tx_json = external_tx_to_json(external_invoke_tx);
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

    // The  `_rx_gateway_to_mempool`   is retained to keep the channel open, as dropping it would
    // prevent the sender from transmitting messages.
    let (tx_gateway_to_mempool, _rx_gateway_to_mempool) = channel::<GatewayToMempoolMessage>(1);
    let (_, rx_mempool_to_gateway) = channel::<MempoolToGatewayMessage>(1);
    let network_component =
        GatewayNetworkComponent::new(tx_gateway_to_mempool, rx_mempool_to_gateway);
    let state_reader_factory = Arc::new(test_state_reader_factory());

    // TODO: Add fixture.
    let gateway = Gateway::new(config, network_component, state_reader_factory);

    let response = gateway.app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), status_code);

    response.into_body().collect().await.unwrap().to_bytes()
}
