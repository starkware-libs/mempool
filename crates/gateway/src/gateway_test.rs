use crate::gateway::add_transaction;
use axum::{
    body::{Bytes, HttpBody},
    response::{IntoResponse, Response},
};
use pretty_assertions::assert_str_eq;
use rstest::rstest;
use starknet_api::core::ChainId;
use starknet_api::external_transaction::ExternalTransaction;
use std::fs::File;
use std::path::Path;

const TEST_FILES_FOLDER: &str = "./tests/fixtures";

// TODO(Ayelet): Replace the use of the JSON files with generated instances, then serialize these
// into JSON for testing.
#[rstest]
#[case::declare(&Path::new(TEST_FILES_FOLDER).join("declare_v3.json"), "0x03822dbc50d129064b16e3ed3ff1af2cb34cdb15f202ea6c5ec98f1cc0190ede")]
#[case::deploy_account(
    &Path::new(TEST_FILES_FOLDER).join("deploy_account_v3.json"),
    "0x0274837d32404e10c2aba2c782854abd692eaed9e4d46676f5611c90de6979f9"
)]
#[case::invoke(&Path::new(TEST_FILES_FOLDER).join("invoke_v3.json"), "0x06401db149315bdd1370b04c911d2e83789017574665744afa257bc1abb00308")]
#[tokio::test]
async fn test_add_transaction(#[case] json_file_path: &Path, #[case] expected_response: &str) {
    use crate::gateway::TransactionInput;

    let json_file = File::open(json_file_path).unwrap();
    let chain_id = ChainId("SN_TEST".to_string());
    let transaction: ExternalTransaction = serde_json::from_reader(json_file).unwrap();
    let input = TransactionInput {
        chain_id,
        transaction,
    };

    let response = add_transaction(input.into()).await.into_response();
    let response_bytes = &to_bytes(response).await;

    assert_str_eq!(&String::from_utf8_lossy(response_bytes), expected_response);
}

async fn to_bytes(res: Response) -> Bytes {
    res.into_body().collect().await.unwrap().to_bytes()
}
