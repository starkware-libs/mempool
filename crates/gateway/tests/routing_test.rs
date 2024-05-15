use std::net::{IpAddr, Ipv4Addr};

use axum::body::{Body, Bytes, HttpBody};
use axum::http::{Request, StatusCode};
use pretty_assertions::assert_str_eq;
use rstest::{fixture, rstest};
use starknet_api::core::{ContractAddress, PatriciaKey};
use starknet_api::external_transaction::ExternalInvokeTransaction;
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{
    Resource, ResourceBounds, ResourceBoundsMapping, TransactionSignature,
};
use starknet_api::{patricia_key, stark_felt};
use starknet_gateway::config::GatewayNetworkConfig;
use starknet_gateway::gateway::Gateway;
use starknet_gateway::invoke_tx_args;
use starknet_gateway::stateless_transaction_validator::StatelessTransactionValidatorConfig;
use starknet_gateway::utils::{external_invoke_tx, external_invoke_tx_to_json};
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

    let (tx_gateway_to_mempool, _rx_gateway_to_mempool) = channel::<GatewayToMempoolMessage>(1);
    let (_, rx_mempool_to_gateway) = channel::<MempoolToGatewayMessage>(1);
    let network_component =
        GatewayNetworkComponent::new(tx_gateway_to_mempool, rx_mempool_to_gateway);

    Gateway { network_config, stateless_transaction_validator_config, network_component }
}

// TODO(Ayelet): add test cases for declare and deploy account transactions.
#[rstest]
#[case::invoke(external_invoke_tx(invoke_tx_args! {
    signature: TransactionSignature(vec![stark_felt!("0x1132577"), stark_felt!("0x17df53c")]),
    contract_address: ContractAddress(patricia_key!(stark_felt!("0x1b34d819720bd84c89bdfb476bc2c4d0de9a41b766efabd20fa292280e4c6d9"))),
    resource_bounds: ResourceBoundsMapping::try_from(vec![
        (
            Resource::L1Gas,
            starknet_api::transaction::ResourceBounds {
                max_amount: 5,
                max_price_per_unit: 6,
            },
        ),
        (
            Resource::L2Gas,
            ResourceBounds {
                max_amount: 0,
                max_price_per_unit: 0,
            },
        ),
    ])
    .unwrap(),
    }), "INVOKE")]
#[tokio::test]
async fn test_routes(
    #[case] external_invoke_tx: ExternalInvokeTransaction,
    #[case] expected_response: &str,
    gateway: Gateway,
) {
    let tx_json = external_invoke_tx_to_json(
        starknet_api::external_transaction::ExternalTransaction::Invoke(external_invoke_tx),
    );
    let request = Request::post("/add_transaction")
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
