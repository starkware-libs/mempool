use std::collections::BTreeMap;
use std::fmt::Debug;
use std::fs::File;
use std::path::Path;

use clap::Command;
use papyrus_config::dumping::SerializeConfig;
use papyrus_config::loading::load_and_process_config;
use papyrus_config::SerializedParam;
use rstest::rstest;
use serde::Deserialize;
use validator::Validate;

use crate::config::{
    GatewayNetworkConfig, RpcStateReaderConfig, StatelessTransactionValidatorConfig,
};

const TEST_FILES_FOLDER: &str = "./src/json_files_for_testing";

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
    fix: bool,
) {
    let path = Path::new(TEST_FILES_FOLDER).join(serialized_config_file_name);

    // Test serialize.
    let dumped_struct = config_struct.dump();
    if fix {
        config_struct.dump_to_file(&vec![], path.to_str().unwrap()).unwrap();
    }
    let file = File::open(path.clone()).unwrap();
    let loaded_file: BTreeMap<String, SerializedParam> = serde_json::from_reader(file).unwrap();
    assert_eq!(dumped_struct, loaded_file);

    // Test deserialize.

    // The function `load_and_process_config` requires the absolute path (from the crate) to the
    // file.
    let config_file = File::open(path).unwrap();
    let loaded_config: T = load_and_process_config(config_file, Command::new(""), vec![]).unwrap();
    assert_eq!(loaded_config, config_struct);

    // Validate the loaded config.
    assert!(config_struct.validate().is_ok());
}

#[rstest]
/// Read the network config file and validate its content.
fn test_network_config(#[values(false)] fix: bool) {
    let config_struct = GatewayNetworkConfig::create_for_testing();
    config_test_body(config_struct, NETWORK_CONFIG, fix);
}

#[rstest]
#[ignore]
/// Fix the config file for test_valid_network_config. Run with 'cargo test -- --ignored'.
fn fix_test_network_config(#[values(true)] fix: bool) {
    let config_struct = GatewayNetworkConfig::create_for_testing();
    config_test_body(config_struct, NETWORK_CONFIG, fix);
}

#[rstest]
/// Read the stateless transaction validator config file and validate its content.
/// fix with "env UPDATE_EXPECT=1 cargo test"
fn test_stateless_transaction_validator_config(#[values(false)] fix: bool) {
    let config_struct = StatelessTransactionValidatorConfig::create_for_testing();
    config_test_body(config_struct, STATELESS_TRANSACTION_VALIDATOR_CONFIG, fix);
}

#[rstest]
#[ignore]
/// Fix the config file for test_valid_stateless_transaction_validator_config.
/// Run with 'cargo test -- --ignored'.
fn fix_test_stateless_transaction_validator_config(#[values(true)] fix: bool) {
    let config_struct = StatelessTransactionValidatorConfig::create_for_testing();
    config_test_body(config_struct, STATELESS_TRANSACTION_VALIDATOR_CONFIG, fix);
}

#[rstest]
/// Read the rpc state reader config file and validate its content.
fn test_valid_rpc_state_reader_config(#[values(false)] fix: bool) {
    let config_struct = RpcStateReaderConfig::create_for_testing();
    config_test_body(config_struct, RPC_STATE_READER_CONFIG, fix);
}

#[rstest]
#[ignore]
/// Fix the config file for test_valid_rpc_state_reader_config.
/// Run with 'cargo test -- --ignored'.
fn fix_test_valid_rpc_state_reader_config(#[values(true)] fix: bool) {
    let config_struct = RpcStateReaderConfig::create_for_testing();
    config_test_body(config_struct, RPC_STATE_READER_CONFIG, fix);
}
