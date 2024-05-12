use std::fmt::Debug;
use std::fs::File;
use std::path::Path;

use clap::Command;
use expect_test::{expect_file, ExpectFile};
use papyrus_config::dumping::SerializeConfig;
use papyrus_config::loading::load_and_process_config;
use serde::Deserialize;
use validator::Validate;

use crate::config::{
    GatewayNetworkConfig, RpcStateReaderConfig, StatelessTransactionValidatorConfig,
};

const TEST_FILES_FOLDER: &str = "json_files_for_testing";
const GATEWAY_SOURCE_FOLDER: &str = "./src";

const NETWORK_CONFIG: &str = "gateway_network_config.json";
const STATELESS_TRANSACTION_VALIDATOR_CONFIG: &str = "stateless_transaction_validator_config.json";
const RPC_STATE_READER_CONFIG: &str = "rpc_state_reader_config.json";

/// Tests the basic functionality of Papyrus config files.
/// Tests the seriazlization of a config struct by calling the 'dump' method, and the
/// deserialization of a config .json file by calling the 'load_and_process_config' function.
/// Finally, it validates the config struct.
fn config_test_body<T: for<'a> Deserialize<'a> + SerializeConfig + Validate + PartialEq + Debug>(
    config_struct: T,
    serialized_config_file_name: &str,
) {
    // Test serialize.

    let serialized_struct = serde_json::to_string_pretty(&config_struct.dump()).unwrap();
    // The macro `expect_file!` requires the relative path to the file.
    let path = Path::new(TEST_FILES_FOLDER).join(serialized_config_file_name);
    let expected_serialized_struct: ExpectFile = expect_file![path.to_str().unwrap()];
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
    assert!(config_struct.validate().is_ok());
}

#[test]
/// Read the network config file and validate its content.
/// Fix with "env UPDATE_EXPECT=1 cargo test"
fn test_network_config() {
    let config_struct = GatewayNetworkConfig { ip: "0.0.0.0".parse().unwrap(), port: 8080 };
    config_test_body(config_struct, NETWORK_CONFIG);
}

#[test]
/// Read the stateless transaction validator config file and validate its content.
/// fix with "env UPDATE_EXPECT=1 cargo test"
fn test_stateless_transaction_validator_config() {
    let config_struct = StatelessTransactionValidatorConfig {
        validate_non_zero_l1_gas_fee: true,
        validate_non_zero_l2_gas_fee: false,
        max_calldata_length: 10,
        max_signature_length: 0,
    };
    config_test_body(config_struct, STATELESS_TRANSACTION_VALIDATOR_CONFIG);
}

#[test]
/// Read the rpc state reader config file and validate its content.
fn test_valid_rpc_state_reader_config() {
    let config_struct = RpcStateReaderConfig::create_for_testing();
    config_test_body(config_struct, RPC_STATE_READER_CONFIG);
}
