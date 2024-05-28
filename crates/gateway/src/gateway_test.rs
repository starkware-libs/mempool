use std::sync::Arc;

use axum::body::{Bytes, HttpBody};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use starknet_mempool::mempool::{create_mempool_server, Mempool};
use starknet_mempool_types::mempool_types::{
    MempoolClient, MempoolInvocation, MempoolRequestAndResponseSender,
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

pub fn app_state(mempool_client: Box<dyn MempoolInvocation>) -> AppState {
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
    // Create and start the mempool server.
    let mempool = Mempool::new([]);
    // TODO(Tsabary): wrap creation of channels in dedicated functions, take channel capacity from
    // config.
    let (tx_mempool, rx_mempool) =
        channel::<MempoolRequestAndResponseSender>(MEMPOOL_INVOCATIONS_QUEUE_SIZE);
    let mut mempool_server = create_mempool_server(mempool, rx_mempool);
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
