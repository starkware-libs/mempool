use std::fmt::Debug;
use std::fs::File;
use std::path::Path;

use clap::Command;
use expect_test::expect_file;
use papyrus_config::dumping::SerializeConfig;
use papyrus_config::loading::load_and_process_config;
use serde::Deserialize;
use validator::Validate;

use crate::config::{GatewayNetworkConfig, StatelessTransactionValidatorConfig};

const TEST_FILES_FOLDER: &str = "json_files_for_testing";
const GATEWAY_SOURCE_FOLDER: &str = "./src";

const NETWORK_CONFIG: &str = "gateway_network_config.json";
const STATELESS_TRANSACTION_VALIDATOR_CONFIG: &str = "stateless_transaction_validator_config.json";

fn config_test_body<T: for<'a> Deserialize<'a> + SerializeConfig + Validate + PartialEq + Debug>(
    config_struct: T,
    serialized_config_file_name: &str,
) {
    // Test serialize.

    // The macro `expect_file!` requires the relative path to the file.
    let path = Path::new(TEST_FILES_FOLDER).join(serialized_config_file_name);
    let expected_serialized_struct = expect_file![path.to_str().unwrap()];
    let serialized_struct = serde_json::to_string_pretty(&config_struct.dump()).unwrap();
    // If the environment variable `UPDATE_EXPECT` is set, the expected file will be updated.
    expected_serialized_struct.assert_eq(&serialized_struct);

    // Test deserialize.

    // The function `load_and_process_config` requires the absolute path (from the crate) to the
    // file.
    let path = Path::new(GATEWAY_SOURCE_FOLDER).join(path);
    let config_file = File::open(path).unwrap();
    let loaded_config: T = load_and_process_config(config_file, Command::new(""), vec![]).unwrap();
    assert_eq!(loaded_config, config_struct);

    // Validate the loaded config.
    assert!(loaded_config.validate().is_ok());
}

#[test]
/// Read the network config file and validate its content.
/// Fix with "env UPDATE_EXPECT=1 cargo test"
fn test_network_config() {
    let expected_config = GatewayNetworkConfig { ip: "0.0.0.0".parse().unwrap(), port: 8080 };
    config_test_body(expected_config, NETWORK_CONFIG);
}

#[test]
/// Read the stateless transaction validator config file and validate its content.
/// fix with "env UPDATE_EXPECT=1 cargo test"
fn test_stateless_transaction_validator_config() {
    let expected_config = StatelessTransactionValidatorConfig {
        validate_non_zero_l1_gas_fee: true,
        validate_non_zero_l2_gas_fee: false,
        max_calldata_length: 10,
        max_signature_length: 0,
    };
    config_test_body(expected_config, STATELESS_TRANSACTION_VALIDATOR_CONFIG);
}
