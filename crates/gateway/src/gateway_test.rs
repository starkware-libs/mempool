use std::sync::Arc;

use axum::body::{Bytes, HttpBody};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use pretty_assertions::assert_str_eq;
use rstest::{fixture, rstest};
use starknet_api::core::{ContractAddress, PatriciaKey};
use starknet_api::external_transaction::{ExternalInvokeTransaction, ExternalTransaction};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{ResourceBounds, TransactionSignature};
use starknet_api::{patricia_key, stark_felt};
use starknet_mempool_types::mempool_types::{
    GatewayNetworkComponent, GatewayToMempoolMessage, MempoolToGatewayMessage,
};
use tokio::sync::mpsc::channel;

use crate::config::StatelessTransactionValidatorConfig;
use crate::gateway::{async_add_tx, AppState};
use crate::invoke_tx_args;
use crate::starknet_api_test_utils::{
    create_resource_bounds_mapping, external_invoke_tx, external_invoke_tx_to_json,
};
use crate::stateless_transaction_validator::StatelessTransactionValidator;

#[fixture]
pub fn network_component() -> GatewayNetworkComponent {
    let (tx_gateway_to_mempool, _rx_gateway_to_mempool) = channel::<GatewayToMempoolMessage>(1);
    let (_, rx_mempool_to_gateway) = channel::<MempoolToGatewayMessage>(1);

    GatewayNetworkComponent::new(tx_gateway_to_mempool, rx_mempool_to_gateway)
}

// TODO(Ayelet): add test cases for declare and deploy account transactions.
#[rstest]
#[case::invoke(
    external_invoke_tx(invoke_tx_args! {
        signature: TransactionSignature(vec![
            stark_felt!("0x1132577"),
            stark_felt!("0x17df53c")
        ]),
        contract_address: ContractAddress(
            patricia_key!(stark_felt!("0x1b34d819720bd84c89bdfb476bc2c4d0de9a41b766efabd20fa292280e4c6d9"))
        ),
        resource_bounds: create_resource_bounds_mapping(
            ResourceBounds {
                max_amount: 5,
                max_price_per_unit: 6,
            },
            ResourceBounds::default()
        )
    }),
    "INVOKE"
)]
#[tokio::test]
async fn test_add_transaction_with_invoke_tx(
    #[case] external_invoke_tx: ExternalInvokeTransaction,
    #[case] expected_response: &str,
    network_component: GatewayNetworkComponent,
) {
    let json_string = external_invoke_tx_to_json(
        starknet_api::external_transaction::ExternalTransaction::Invoke(external_invoke_tx),
    );
    let tx: ExternalTransaction = serde_json::from_str(&json_string).unwrap();

    let mut app_state = AppState {
        stateless_transaction_validator: StatelessTransactionValidator {
            config: StatelessTransactionValidatorConfig {
                validate_non_zero_l1_gas_fee: true,
                max_calldata_length: 10,
                ..Default::default()
            },
        },
        network_component: Arc::new(network_component),
    };

    // Negative flow.
    const TOO_SMALL_SIGNATURE_LENGTH: usize = 0;
    app_state.stateless_transaction_validator.config.max_signature_length =
        TOO_SMALL_SIGNATURE_LENGTH;

    let response = async_add_tx(State(app_state.clone()), tx.clone().into()).await.into_response();

    let status_code = response.status();
    assert_eq!(status_code, StatusCode::INTERNAL_SERVER_ERROR);

    let response_bytes = &to_bytes(response).await;
    let negative_flow_expected_response = "Signature length exceeded maximum:";
    assert!(String::from_utf8_lossy(response_bytes).starts_with(negative_flow_expected_response));

    // Positive flow.
    app_state.stateless_transaction_validator.config.max_signature_length = 2;

    let response = async_add_tx(State(app_state), tx.into()).await.into_response();

    let status_code = response.status();
    assert_eq!(status_code, StatusCode::OK);

    let response_bytes = &to_bytes(response).await;
    assert_str_eq!(&String::from_utf8_lossy(response_bytes), expected_response);
}

async fn to_bytes(res: Response) -> Bytes {
    res.into_body().collect().await.unwrap().to_bytes()
}
