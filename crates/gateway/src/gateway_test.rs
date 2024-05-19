use std::sync::Arc;

use axum::body::{Bytes, HttpBody};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use blockifier::test_utils::contracts::FeatureContract;
use blockifier::test_utils::{create_trivial_calldata, CairoVersion, NonceManager};
use pretty_assertions::assert_str_eq;
use rstest::rstest;
use starknet_api::external_transaction::ExternalTransaction;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::TransactionSignature;
use starknet_mempool_types::mempool_types::{
    GatewayNetworkComponent, GatewayToMempoolMessage, MempoolToGatewayMessage,
};
use tokio::sync::mpsc::channel;

use crate::config::StatelessTransactionValidatorConfig;
use crate::gateway::{add_tx, AppState};
use crate::starknet_api_test_utils::{
    executable_external_invoke_tx_for_testing, executable_resource_bounds_mapping,
};
use crate::state_reader_test_utils::test_state_reader_factory;
use crate::stateful_transaction_validator::{
    StatefulTransactionValidator, StatefulTransactionValidatorConfig,
};
use crate::stateless_transaction_validator::StatelessTransactionValidator;

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
async fn test_add_tx(#[case] tx: ExternalTransaction, #[case] expected_response: &str) {
    // The  `_rx_gateway_to_mempool`   is retained to keep the channel open, as dropping it would
    // prevent the sender from transmitting messages.
    let (tx_gateway_to_mempool, _rx_gateway_to_mempool) = channel::<GatewayToMempoolMessage>(1);
    let (_, rx_mempool_to_gateway) = channel::<MempoolToGatewayMessage>(1);

    // TODO: Add fixture.
    let network_component =
        Arc::new(GatewayNetworkComponent::new(tx_gateway_to_mempool, rx_mempool_to_gateway));

    let mut app_state = AppState {
        stateless_transaction_validator: StatelessTransactionValidator {
            config: StatelessTransactionValidatorConfig {
                validate_non_zero_l1_gas_fee: true,
                max_calldata_length: 10,
                ..Default::default()
            },
        },
        network_component,
        stateful_transaction_validator: Arc::new(StatefulTransactionValidator {
            config: StatefulTransactionValidatorConfig::create_for_testing(),
        }),
        state_reader_factory: Arc::new(test_state_reader_factory()),
    };

    // Negative flow.
    const TOO_SMALL_SIGNATURE_LENGTH: usize = 0;
    app_state.stateless_transaction_validator.config.max_signature_length =
        TOO_SMALL_SIGNATURE_LENGTH;

    let response = add_tx(State(app_state.clone()), tx.clone().into()).await.into_response();

    let status_code = response.status();
    assert_eq!(status_code, StatusCode::INTERNAL_SERVER_ERROR);

    let response_bytes = &to_bytes(response).await;
    let negative_flow_expected_response = "Signature length exceeded maximum:";
    assert!(String::from_utf8_lossy(response_bytes).starts_with(negative_flow_expected_response));

    // Positive flow.
    app_state.stateless_transaction_validator.config.max_signature_length = 2;

    let response = add_tx(State(app_state), tx.into()).await.into_response();

    let status_code = response.status();
    assert_eq!(status_code, StatusCode::OK);

    let response_bytes = &to_bytes(response).await;
    assert_str_eq!(&String::from_utf8_lossy(response_bytes), expected_response);
}

async fn to_bytes(res: Response) -> Bytes {
    res.into_body().collect().await.unwrap().to_bytes()
}
