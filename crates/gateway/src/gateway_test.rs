use std::sync::Arc;

use axum::body::{Bytes, HttpBody};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use rstest::rstest;
use starknet_mempool_types::mempool_types::{
    GatewayNetworkComponent, GatewayToMempoolMessage, MempoolToGatewayMessage,
};
use tokio::sync::mpsc::channel;

use crate::config::{StatefulTransactionValidatorConfig, StatelessTransactionValidatorConfig};
use crate::gateway::{add_tx, AppState};
use crate::starknet_api_test_utils::invoke_tx;
use crate::state_reader_test_utils::test_state_reader_factory;
use crate::stateful_transaction_validator::StatefulTransactionValidator;
use crate::stateless_transaction_validator::StatelessTransactionValidator;

pub fn app_state(
    gateway_component: GatewayNetworkComponent,
    max_signature_length: usize,
) -> AppState {
    AppState {
        stateless_transaction_validator: StatelessTransactionValidator {
            config: StatelessTransactionValidatorConfig {
                validate_non_zero_l1_gas_fee: true,
                max_calldata_length: 10,
                max_signature_length,
                ..Default::default()
            },
        },
        network_component: Arc::new(gateway_component),
        stateful_transaction_validator: Arc::new(StatefulTransactionValidator {
            config: StatefulTransactionValidatorConfig::create_for_testing(),
        }),
        state_reader_factory: Arc::new(test_state_reader_factory()),
    }
}

// TODO(Ayelet): add test cases for declare and deploy account transactions.
#[rstest]
#[case::positive(2, StatusCode::OK, "INVOKE")]
#[case::negative(0, StatusCode::INTERNAL_SERVER_ERROR, "Signature length exceeded maximum:")]
#[tokio::test]
async fn test_add_tx(
    #[case] max_signature_length: usize,
    #[case] expected_status_code: StatusCode,
    #[case] expected_response: &str,
) {
    // The  `_rx_gateway_to_mempool`   is retained to keep the channel open, as dropping it would
    // prevent the sender from transmitting messages.
    let (tx_gateway_to_mempool, _rx_gateway_to_mempool) = channel::<GatewayToMempoolMessage>(1);
    let (_, rx_mempool_to_gateway) = channel::<MempoolToGatewayMessage>(1);
    // TODO: Add fixture.
    let gateway_component =
        GatewayNetworkComponent::new(tx_gateway_to_mempool, rx_mempool_to_gateway);

    let app_state = app_state(gateway_component, max_signature_length);

    let response =
        add_tx(State(app_state.clone()), invoke_tx().clone().into()).await.into_response();

    let status_code = response.status();
    assert_eq!(status_code, expected_status_code);

    let response_bytes = &to_bytes(response).await;
    assert!(String::from_utf8_lossy(response_bytes).starts_with(expected_response));
}

async fn to_bytes(res: Response) -> Bytes {
    res.into_body().collect().await.unwrap().to_bytes()
}
