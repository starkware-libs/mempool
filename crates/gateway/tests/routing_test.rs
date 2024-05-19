#![cfg(feature = "testing")]
use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;
use std::sync::Arc;

use axum::body::{Body, Bytes, HttpBody};
use axum::http::{Request, StatusCode};
use blockifier::test_utils::contracts::FeatureContract;
use blockifier::test_utils::{create_trivial_calldata, CairoVersion, NonceManager};
use pretty_assertions::assert_str_eq;
use rstest::{fixture, rstest};
use starknet_api::external_transaction::ExternalTransaction;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::TransactionSignature;
use starknet_gateway::config::{GatewayNetworkConfig, StatelessTransactionValidatorConfig};
use starknet_gateway::gateway::Gateway;
use starknet_gateway::starknet_api_test_utils::{
    executable_external_invoke_tx_for_testing, executable_resource_bounds_mapping,
};
use starknet_gateway::state_reader_test_utils::{
    test_state_reader_factory, TestStateReader, TestStateReaderFactory,
};
use starknet_gateway::stateful_transaction_validator::StatefulTransactionValidatorConfig;
use starknet_mempool_types::mempool_types::{
    GatewayNetworkComponent, GatewayToMempoolMessage, MempoolToGatewayMessage,
};
use tokio::sync::mpsc::channel;
use tower::ServiceExt;

#[fixture]
pub fn gateway() -> Gateway {
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
    let state_reader_factory = test_state_reader_factory();

    let (tx_gateway_to_mempool, _rx_gateway_to_mempool) = channel::<GatewayToMempoolMessage>(1);
    let (_, rx_mempool_to_gateway) = channel::<MempoolToGatewayMessage>(1);
    let network_component =
        GatewayNetworkComponent::new(tx_gateway_to_mempool, rx_mempool_to_gateway);

    Gateway {
        network_config,
        stateless_transaction_validator_config,
        stateful_transaction_validator_config,
        network_component,
        state_reader_factory: Arc::new(state_reader_factory),
    }
}

// TODO(Yael, 19/5/2024): refactor testing infrastructure to genereate a consistent state and
// transaction for all tests in one place
pub fn invoke_tx() -> ExternalTransaction {
    let cairo_version = CairoVersion::Cairo1;
    let account_contract = FeatureContract::AccountWithoutValidations(cairo_version);
    let account_address = account_contract.get_instance_address(0);
    let test_contract = FeatureContract::TestContract(cairo_version);
    let test_contract_address = test_contract.get_instance_address(0);
    let calldata = create_trivial_calldata(test_contract_address);
    let mut nonce_manager = NonceManager::default();
    let nonce = nonce_manager.next(account_address);
    executable_external_invoke_tx_for_testing(
        executable_resource_bounds_mapping(),
        nonce,
        account_address,
        calldata,
        TransactionSignature(vec![StarkFelt::ZERO]),
    )
}

// TODO(Ayelet): Replace the use of the JSON files with generated instances, then serialize these
// into JSON for testing.
#[rstest]
// TODO (Yael 19/5/2024): Add declare and deploy_account in the next milestone
#[case::invoke(invoke_tx(), "INVOKE")]
#[tokio::test]
async fn test_routes(
    #[case] tx: ExternalTransaction,
    #[case] expected_response: &str,
    gateway: Gateway,
) {
    let tx_json = serde_json::to_string(&tx).unwrap();
    let request = Request::post("/add_tx")
        .header("content-type", "application/json")
        .body(Body::from(tx_json))
        .unwrap();

    let response = check_request(request, StatusCode::OK, gateway).await;

    assert_str_eq!(expected_response, String::from_utf8_lossy(&response));
}

#[rstest]
#[tokio::test]
#[should_panic]
// FIXME: Currently is_alive is not implemented, fix this once it is implemented.
async fn test_is_alive(gateway: Gateway) {
    let request = Request::get("/is_alive").body(Body::empty()).unwrap();
    // Status code doesn't matter, this panics ATM.
    check_request(request, StatusCode::default(), gateway).await;
}

async fn check_request(request: Request<Body>, status_code: StatusCode, gateway: Gateway) -> Bytes {
    let response = gateway.app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), status_code);

    response.into_body().collect().await.unwrap().to_bytes()
}
