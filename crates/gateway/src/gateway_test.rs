use std::sync::Arc;

use axum::body::{Bytes, HttpBody};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use mempool_infra::component_server::ComponentServer;
use starknet_mempool::mempool::{Mempool, MempoolCommunicationWrapper};
use starknet_mempool_types::mempool_types::{
    BatcherToMempoolChannels, BatcherToMempoolMessage, GatewayToMempoolMessage, MempoolClient,
    MempoolInterface, MempoolMessageAndResponseSender, MempoolNetworkComponent,
    MempoolToBatcherMessage, MempoolToGatewayMessage,
};
use tokio::sync::mpsc::channel;
use tokio::task;

use crate::config::{StatefulTransactionValidatorConfig, StatelessTransactionValidatorConfig};
use crate::gateway::{add_tx, AppState};
use crate::starknet_api_test_utils::invoke_tx;
use crate::state_reader_test_utils::test_state_reader_factory;
use crate::stateful_transaction_validator::StatefulTransactionValidator;
use crate::stateless_transaction_validator::StatelessTransactionValidator;

const MEMPOOL_INVOCATIONS_QUEUE_SIZE: usize = 32;

pub fn app_state(mempool_client: Box<dyn MempoolInterface>) -> AppState {
    AppState {
        stateless_transaction_validator: StatelessTransactionValidator {
            config: StatelessTransactionValidatorConfig {
                validate_non_zero_l1_gas_fee: true,
                max_calldata_length: 10,
                max_signature_length: 2,
                ..Default::default()
            },
        },
        stateful_transaction_validator: Arc::new(StatefulTransactionValidator {
            config: StatefulTransactionValidatorConfig::create_for_testing(),
        }),
        state_reader_factory: Arc::new(test_state_reader_factory()),
        mempool: Arc::new(mempool_client),
    }
}

// TODO(Ayelet): add test cases for declare and deploy account transactions.
#[tokio::test]
async fn test_add_tx() {
    // The `_rx_gateway_to_mempool` is retained to keep the channel open, as dropping it would
    // prevent the sender from transmitting messages.
    let (_, _rx_gateway_to_mempool) = channel::<GatewayToMempoolMessage>(1);

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
    // TODO(Tsabary): wrap creation of channels in dedicated functions, take channel capacity from
    // config.
    let (tx_mempool, rx_mempool) =
        channel::<MempoolMessageAndResponseSender>(MEMPOOL_INVOCATIONS_QUEUE_SIZE);
    // TODO(Tsabary, 1/6/2024): Wrap with a dedicated create_mempool_server function.
    let mut mempool_server =
        ComponentServer::new(MempoolCommunicationWrapper { mempool }, rx_mempool);
    task::spawn(async move {
        mempool_server.start().await;
    });

    let mempool_client = Box::new(MempoolClient::new(tx_mempool));

    let app_state = app_state(mempool_client);

    let response = add_tx(State(app_state), invoke_tx().into()).await.into_response();

    let status_code = response.status();
    assert_eq!(status_code, StatusCode::OK);

    let response_bytes = &to_bytes(response).await;
    assert!(String::from_utf8_lossy(response_bytes).starts_with("INVOKE"));
}

async fn to_bytes(res: Response) -> Bytes {
    res.into_body().collect().await.unwrap().to_bytes()
}
