use std::sync::Arc;

use axum::body::{Bytes, HttpBody};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use blockifier::blockifier::block::BlockInfo;
use blockifier::test_utils::dict_state_reader::DictStateReader;
use rstest::{fixture, rstest};
use starknet_api::core::{ContractAddress, PatriciaKey};
use starknet_api::external_transaction::ExternalTransaction;
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{ResourceBounds, TransactionSignature};
use starknet_api::{patricia_key, stark_felt};
use starknet_mempool_types::mempool_types::{
    GatewayNetworkComponent, GatewayToMempoolMessage, MempoolToGatewayMessage,
};
use tokio::sync::mpsc::channel;

use crate::config::StatelessTransactionValidatorConfig;
use crate::gateway::{add_tx, AppState};
use crate::invoke_tx_args;
use crate::starknet_api_test_utils::{
    create_resource_bounds_mapping, external_invoke_tx, external_invoke_tx_to_json,
};
use crate::state_reader_test_utils::{TestStateReader, TestStateReaderFactory};
use crate::stateful_transaction_validator::{
    StatefulTransactionValidator, StatefulTransactionValidatorConfig,
};
use crate::stateless_transaction_validator::StatelessTransactionValidator;

#[fixture]
pub fn invoke_tx() -> ExternalTransaction {
    external_invoke_tx(invoke_tx_args! {
        signature: TransactionSignature(vec![stark_felt!("0x1132577"), stark_felt!("0x17df53c")]),
        contract_address: ContractAddress(patricia_key!(stark_felt!("0x1b34d819720bd84c89bdfb476bc2c4d0de9a41b766efabd20fa292280e4c6d9"))),
        resource_bounds: create_resource_bounds_mapping(
            ResourceBounds {max_amount: 5, max_price_per_unit: 6},
            ResourceBounds::default()
        )
    })
}

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
        state_reader_factory: Arc::new(TestStateReaderFactory {
            state_reader: TestStateReader {
                block_info: BlockInfo::create_for_testing(),
                // TODO(yael 16/5/2024): create a test state that will make the tx pass validations
                blockifier_state_reader: DictStateReader::default(),
            },
        }),
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
    // The `_rx_gateway_to_mempool` is retained to keep the channel open, as dropping it would
    // prevent the sender from transmitting messages.
    let (tx_gateway_to_mempool, _rx_gateway_to_mempool) = channel::<GatewayToMempoolMessage>(1);
    let (_, rx_mempool_to_gateway) = channel::<MempoolToGatewayMessage>(1);
    let network_component =
        GatewayNetworkComponent::new(tx_gateway_to_mempool, rx_mempool_to_gateway);

    let app_state = app_state(network_component, max_signature_length);

    let json_string = external_invoke_tx_to_json(invoke_tx());
    let tx: ExternalTransaction = serde_json::from_str(&json_string).unwrap();

    let response = add_tx(State(app_state), tx.clone().into()).await.into_response();

    let status_code = response.status();
    assert_eq!(status_code, expected_status_code);

    let response_bytes = &to_bytes(response).await;
    assert!(String::from_utf8_lossy(response_bytes).starts_with(expected_response));
}

async fn to_bytes(res: Response) -> Bytes {
    res.into_body().collect().await.unwrap().to_bytes()
}
