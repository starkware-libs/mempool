use std::fmt::Debug;
use std::fs::File;
use std::path::{Path, PathBuf};

use clap::Command;
use expect_test::expect_file;
use papyrus_config::dumping::SerializeConfig;
use papyrus_config::loading::load_and_process_config;
use serde::Deserialize;
use validator::Validate;

use crate::config::GatewayNetworkConfig;
use crate::stateless_transaction_validator::StatelessTransactionValidatorConfig;

const TEST_FILES_FOLDER: &str = "json_files_for_testing";
const GATEWAY_SOURCE_FOLDER: &str = "./src";
const NETWORK_CONFIG_FILE: &str = "gateway_network_config.json";
const STATELESS_TRANSACTION_VALIDATOR_CONFIG: &str = "stateless_transaction_validator_config.json";

fn get_config_from_file<T: for<'a> Deserialize<'a>>(
    file_path: PathBuf,
) -> Result<T, papyrus_config::ConfigError> {
    let config_file = File::open(file_path).unwrap();
    load_and_process_config(config_file, Command::new(""), vec![])
}

fn test_valid_config_body<
    T: for<'a> Deserialize<'a> + SerializeConfig + Validate + PartialEq + Debug,
>(
    config_struct: T,
    file_name: &str,
) {
    let path = Path::new(TEST_FILES_FOLDER).join(file_name);
    // Test seralize.
    let expected_file_content = expect_file![path.to_str().unwrap()];
    let give_file_content = serde_json::to_string_pretty(&config_struct.dump()).unwrap();
    expected_file_content.assert_eq(&give_file_content);

    // Test deserialize.
    let path = Path::new(GATEWAY_SOURCE_FOLDER).join(path);
    // let file_path_to_load = Path::new(TEST_FILES_FOLDER).join(file_name);
    let loaded_config: T = get_config_from_file(path).unwrap();

    // Validate the loaded config.
    assert!(loaded_config.validate().is_ok());
    assert_eq!(loaded_config, config_struct);
}

#[test]
/// Read the network config file and validate its content.
/// Fix with "env UPDATE_EXPECT=1 cargo test"
fn test_valid_network_config() {
    let expected_config = GatewayNetworkConfig { ip: "0.0.0.0".parse().unwrap(), port: 8080 };
    test_valid_config_body(expected_config, NETWORK_CONFIG_FILE);
}

#[test]
/// Read the stateless transaction validator config file and validate its content.
/// fix with "env UPDATE_EXPECT=1 cargo test"
fn test_valid_stateless_transaction_validator_config() {
    let expected_config = StatelessTransactionValidatorConfig {
        validate_non_zero_l1_gas_fee: true,
        validate_non_zero_l2_gas_fee: false,
        max_calldata_length: 10,
        max_signature_length: 0,
    };
    test_valid_config_body(expected_config, STATELESS_TRANSACTION_VALIDATOR_CONFIG);
}
