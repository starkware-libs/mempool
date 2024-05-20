use std::sync::Arc;

use axum::body::{Bytes, HttpBody};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use mempool_infra::component_client::ComponentClient;
use mempool_infra::component_server::{ComponentServer, MessageAndResponseSender};
use pretty_assertions::assert_str_eq;
use rstest::rstest;
use starknet_api::external_transaction::ExternalTransaction;
use starknet_mempool::mempool::Mempool;
use starknet_mempool_types::mempool_types::{
    BatcherToMempoolChannels, BatcherToMempoolMessage, GatewayNetworkComponent,
    GatewayToMempoolMessage, MempoolMessages, MempoolNetworkComponent, MempoolResponses,
    MempoolToBatcherMessage, MempoolToGatewayMessage,
};
use tokio::sync::mpsc::channel;
use tokio::sync::Mutex;
use tokio::task;

use crate::config::{StatefulTransactionValidatorConfig, StatelessTransactionValidatorConfig};
use crate::gateway::{add_tx, AppState};
use crate::starknet_api_test_utils::invoke_tx;
use crate::state_reader_test_utils::test_state_reader_factory;
use crate::stateful_transaction_validator::StatefulTransactionValidator;
use crate::stateless_transaction_validator::StatelessTransactionValidator;

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

    // TODO -- remove gateway_network, batcher_network, and channels.
    let (_, rx_gateway_to_mempool) = channel::<GatewayToMempoolMessage>(1);
    let (tx_mempool_to_gateway, _) = channel::<MempoolToGatewayMessage>(1);
    let gateway_network =
        MempoolNetworkComponent::new(tx_mempool_to_gateway, rx_gateway_to_mempool);

    let (_, rx_mempool_to_batcher) = channel::<BatcherToMempoolMessage>(1);
    let (tx_batcher_to_mempool, _) = channel::<MempoolToBatcherMessage>(1);
    let batcher_network =
        BatcherToMempoolChannels { rx: rx_mempool_to_batcher, tx: tx_batcher_to_mempool };

    // Create and start the mempool server.
    let mempool = Mempool::new([], gateway_network, batcher_network);
    let (tx_mempool, rx_mempool) =
        channel::<MessageAndResponseSender<MempoolMessages, MempoolResponses>>(32);
    let mut mempool_server = ComponentServer::new(mempool, rx_mempool);
    task::spawn(async move {
        mempool_server.start().await;
    });

    // Create the gateway's mempool client.
    let gateway_mempool_client =
        Box::new(ComponentClient::<MempoolMessages, MempoolResponses>::new(tx_mempool.clone()));

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
        mempool: Arc::new(Mutex::new(gateway_mempool_client)),
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
