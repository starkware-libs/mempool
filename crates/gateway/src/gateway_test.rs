use crate::gateway::add_transaction;
use crate::GatewayConfig;
use axum::{body::HttpBody, response::IntoResponse};
use clap::Command;
use papyrus_config::loading::load_and_process_config;
use rstest::rstest;
use starknet_api::external_transaction::ExternalTransaction;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use validator::Validate;

// TODO(Ayelet): Replace the use of the JSON files with generated instances, then serialize these
// into JSON for testing.
#[rstest]
#[case("./src/json_files_for_testing/declare_v3.json", "DECLARE")]
#[case(
    "./src/json_files_for_testing/deploy_account_v3.json",
    "DEPLOY_ACCOUNT"
)]
#[case("./src/json_files_for_testing/invoke_v3.json", "INVOKE")]
#[tokio::test]
async fn test_add_transaction(#[case] json_file_path: &str, #[case] expected_response: &str) {
    let file = File::open(json_file_path).unwrap();
    let reader = BufReader::new(file);
    let transaction: ExternalTransaction = serde_json::from_reader(reader).unwrap();
    let response = add_transaction(transaction.into()).await.into_response();
    let response_bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(
        &String::from_utf8(response_bytes.to_vec()).unwrap(),
        expected_response
    );
}

const DEFAULT_GOOD_CONFIG_PATH: &str = "./src/json_files_for_testing/good_gateway_config.json";
const DEFAULT_BAD_CONFIG_PATH: &str = "./src/json_files_for_testing/bad_gateway_config.json";
const DEFAULT_BAD_ADDRESS_CONFIG_PATH: &str =
    "./src/json_files_for_testing/bad_gateway_address_config.json";

#[test]
fn good_config_test() {
    // Read the good config file and validate it.
    let config = GatewayConfig::default();

    let config_file = File::open(Path::new(DEFAULT_GOOD_CONFIG_PATH)).unwrap();
    let load_config =
        load_and_process_config::<GatewayConfig>(config_file, Command::new(""), vec![]).unwrap();
    assert!(load_config.validate().is_ok());
    assert_eq!(load_config.bind_address, config.bind_address);
}

#[test]
fn bad_config_test() {
    // Read the config file with the bad field path and validate it.
    let config_file = std::fs::File::open(Path::new(DEFAULT_BAD_CONFIG_PATH)).unwrap();
    let load_config =
        load_and_process_config::<GatewayConfig>(config_file, Command::new(""), vec![]);
    match load_config {
        Ok(_) => panic!("Expected an error, but got a config."),
        Err(e) => assert_eq!(e.to_string(), "missing field `bind_address`".to_owned()),
    }
}

#[test]
fn bad_config_address_test() {
    // Read the config file with the bad bind_address values and validate it
    let config_file = std::fs::File::open(Path::new(DEFAULT_BAD_ADDRESS_CONFIG_PATH)).unwrap();
    let load_config =
        load_and_process_config::<GatewayConfig>(config_file, Command::new(""), vec![]).unwrap();
    match load_config.validate() {
        Ok(_) => panic!("Expected an error, but got a config."),
        Err(e) => assert_eq!(
            e.field_errors()["bind_address"][0].code,
            "Invalid Socket address.".to_owned()
        ),
    }
}
