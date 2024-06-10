use std::sync::Arc;

use axum::body::{Bytes, HttpBody};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use blockifier::context::ChainInfo;
use rstest::rstest;
use starknet_api::external_transaction::ExternalTransaction;
use starknet_api::transaction::TransactionHash;
use starknet_mempool::mempool::{create_mempool_server, Mempool};
use starknet_mempool_types::mempool_types::{
    MempoolClient, MempoolClientImpl, MempoolRequestAndResponseSender,
};
use tokio::sync::mpsc::channel;
use tokio::task;

use crate::config::{StatefulTransactionValidatorConfig, StatelessTransactionValidatorConfig};
use crate::gateway::{add_tx, get_optional_class_info, AppState};
use crate::starknet_api_test_utils::{declare_tx, invoke_tx};
use crate::state_reader_test_utils::local_test_state_reader_factory;
use crate::stateful_transaction_validator::StatefulTransactionValidator;
use crate::stateless_transaction_validator::StatelessTransactionValidator;
use crate::utils::{external_tx_to_account_tx, get_tx_hash};

const MEMPOOL_INVOCATIONS_QUEUE_SIZE: usize = 32;

pub fn app_state(mempool_client: Arc<dyn MempoolClient>) -> AppState {
    AppState {
        stateless_tx_validator: StatelessTransactionValidator {
            config: StatelessTransactionValidatorConfig {
                validate_non_zero_l1_gas_fee: true,
                max_calldata_length: 10,
                max_signature_length: 2,
                ..Default::default()
            },
        },
        stateful_tx_validator: Arc::new(StatefulTransactionValidator {
            config: StatefulTransactionValidatorConfig::create_for_testing(),
        }),
        state_reader_factory: Arc::new(local_test_state_reader_factory()),
        mempool_client,
    }
}

// TODO(Ayelet): add test cases for declare and deploy account transactions.
#[tokio::test]
#[rstest]
async fn test_add_tx(#[values(declare_tx(), invoke_tx())] tx: ExternalTransaction) {
    // TODO: Add fixture.

    let mempool = Mempool::new([]);
    // TODO(Tsabary): wrap creation of channels in dedicated functions, take channel capacity from
    // config.
    let (tx_mempool, rx_mempool) =
        channel::<MempoolRequestAndResponseSender>(MEMPOOL_INVOCATIONS_QUEUE_SIZE);
    let mut mempool_server = create_mempool_server(mempool, rx_mempool);
    task::spawn(async move {
        mempool_server.start().await;
    });

    let mempool_client = Arc::new(MempoolClientImpl::new(tx_mempool));

    let app_state = app_state(mempool_client);

    // Scenario based test.
    let tx_hash = calculate_hash(&tx);
    let response = add_tx(State(app_state), tx.into()).await.into_response();

    let status_code = response.status();
    let response_bytes = &to_bytes(response).await;

    assert_eq!(status_code, StatusCode::OK, "{response_bytes:?}");
    assert_eq!(tx_hash, serde_json::from_slice(response_bytes).unwrap());
}

async fn to_bytes(res: Response) -> Bytes {
    res.into_body().collect().await.unwrap().to_bytes()
}

fn calculate_hash(external_tx: &ExternalTransaction) -> TransactionHash {
    match external_tx {
        ExternalTransaction::Invoke(_) | ExternalTransaction::Declare(_) => {}
        _ => {
            panic!("Only Declare supported for now, extend as needed.");
        }
    }

    let optional_class_info = get_optional_class_info(external_tx).unwrap();

    let account_tx = external_tx_to_account_tx(
        external_tx,
        optional_class_info,
        &ChainInfo::create_for_testing().chain_id,
    )
    .unwrap();
    get_tx_hash(&account_tx)
}
