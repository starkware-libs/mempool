use axum::{
    body::{Bytes, HttpBody},
    response::{IntoResponse, Response},
};
use pretty_assertions::assert_str_eq;
use rstest::rstest;
use starknet_api::external_transaction::ExternalTransaction;
use std::fs::File;

use crate::gateway;

// TODO(Ayelet): Replace the use of the JSON files with generated instances, then serialize these
// into JSON for testing.
#[rstest]
#[case(
    "./src/json_files_for_testing/declare_v3.json",
    "0x03822dbc50d129064b16e3ed3ff1af2cb34cdb15f202ea6c5ec98f1cc0190ede"
)]
#[case(
    "./src/json_files_for_testing/deploy_account_v3.json",
    "0x0274837d32404e10c2aba2c782854abd692eaed9e4d46676f5611c90de6979f9"
)]
#[case(
    "./src/json_files_for_testing/invoke_v3.json",
    "0x06401db149315bdd1370b04c911d2e83789017574665744afa257bc1abb00308"
)]
#[tokio::test]
async fn test_add_transaction(#[case] json_file_path: &str, #[case] expected_response: &str) {
    use starknet_api::core::ChainId;

    let json_file = File::open(json_file_path).unwrap();
    let tx: ExternalTransaction = serde_json::from_reader(json_file).unwrap();

    let gateway = gateway::Gateway {
        config: gateway::GatewayConfig {
            bind_address: "0.0.0.0:8080".to_string(),
        },
        chain_id: ChainId("SN_TEST".to_string()),
    };

    let response = gateway.add_transaction(tx.into()).await.into_response();
    let response_bytes = &to_bytes(response).await;

    assert_str_eq!(&String::from_utf8_lossy(response_bytes), expected_response);
}

async fn to_bytes(res: Response) -> Bytes {
    res.into_body().collect().await.unwrap().to_bytes()
}
