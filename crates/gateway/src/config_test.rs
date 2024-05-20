use std::fmt::Debug;
use std::fs::File;
use std::path::{Path, PathBuf};

use clap::Command;
use papyrus_config::dumping::SerializeConfig;
use papyrus_config::loading::load_and_process_config;
use rstest::{fixture, rstest};
use serde::Deserialize;
use validator::Validate;

use crate::config::{
    GatewayConfig, GatewayNetworkConfig, RpcStateReaderConfig, StatelessTransactionValidatorConfig,
};

const TEST_FILES_FOLDER: &str = "./src/json_files_for_testing";
const NETWORK_CONFIG_FILE: &str = "gateway_network_config.json";
const STATELESS_TRANSACTION_VALIDATOR_CONFIG: &str = "stateless_transaction_validator_config.json";
const RPC_STATE_READER_CONFIG: &str = "rpc_state_reader_config.json";
const GATEWAY_CONFIG_FILE: &str = "gateway_config.json";

fn get_config_file_path(file_name: &str) -> PathBuf {
    Path::new(TEST_FILES_FOLDER).join(file_name)
}

fn get_config_from_file<T: for<'a> Deserialize<'a>>(
    file_path: PathBuf,
) -> Result<T, papyrus_config::ConfigError> {
    let config_file = File::open(file_path).unwrap();
    load_and_process_config(config_file, Command::new(""), vec![])
}

fn test_valid_config_body<
    T: for<'a> Deserialize<'a> + SerializeConfig + Validate + PartialEq + Debug,
>(
    expected_config: T,
    config_file_path: PathBuf,
    fix: bool,
) {
    if fix {
        expected_config.dump_to_file(&vec![], config_file_path.to_str().unwrap()).unwrap();
    }

    let loaded_config: T = get_config_from_file(config_file_path).unwrap();

    assert!(loaded_config.validate().is_ok());
    assert_eq!(loaded_config, expected_config);
}

#[fixture]
fn gateway_network_config() -> GatewayNetworkConfig {
    GatewayNetworkConfig { ip: "0.0.0.0".parse().unwrap(), port: 8080 }
}

#[fixture]
fn stateless_transaction_validator_config() -> StatelessTransactionValidatorConfig {
    StatelessTransactionValidatorConfig {
        validate_non_zero_l1_gas_fee: true,
        validate_non_zero_l2_gas_fee: false,
        max_calldata_length: 10,
        max_signature_length: 0,
    }
}

#[fixture]
fn rpc_state_reader_config() -> RpcStateReaderConfig {
    RpcStateReaderConfig {
        url: "http://localhost:8080".to_string(),
        json_rpc_version: "2.0".to_string(),
    }
}

#[fixture]
fn gateway_config(
    gateway_network_config: GatewayNetworkConfig,
    stateless_transaction_validator_config: StatelessTransactionValidatorConfig,
) -> GatewayConfig {
    GatewayConfig { network_config: gateway_network_config, stateless_transaction_validator_config }
}

#[rstest]
/// Read the network config file and validate its content.
fn test_valid_network_config(gateway_network_config: GatewayNetworkConfig) {
    let file_path = get_config_file_path(NETWORK_CONFIG_FILE);
    let fix = false;
    test_valid_config_body(gateway_network_config, file_path, fix);
}

// TODO(Arni, 7/5/2024): Dedup code with test_valid_config.
#[rstest]
#[ignore]
/// Fix the config file for test_valid_network_config. Run with 'cargo test -- --ignored'.
fn fix_test_valid_network_config(gateway_network_config: GatewayNetworkConfig) {
    let file_path = get_config_file_path(NETWORK_CONFIG_FILE);
    let fix = true;
    test_valid_config_body(gateway_network_config, file_path, fix);
}

#[rstest]
/// Read the stateless transaction validator config file and validate its content.
fn test_valid_stateless_transaction_validator_config(
    stateless_transaction_validator_config: StatelessTransactionValidatorConfig,
) {
    let file_path = get_config_file_path(STATELESS_TRANSACTION_VALIDATOR_CONFIG);
    let fix = false;
    test_valid_config_body(stateless_transaction_validator_config, file_path, fix);
}

#[rstest]
#[ignore]
/// Fix the config file for test_valid_stateless_transaction_validator_config.
/// Run with 'cargo test -- --ignored'.
fn fix_test_valid_stateless_transaction_validator_config(
    stateless_transaction_validator_config: StatelessTransactionValidatorConfig,
) {
    let file_path = get_config_file_path(STATELESS_TRANSACTION_VALIDATOR_CONFIG);
    let fix = true;
    test_valid_config_body(stateless_transaction_validator_config, file_path, fix);
}

#[rstest]
/// Read the rpc state reader config file and validate its content.
fn test_valid_rpc_state_reader_config(rpc_state_reader_config: RpcStateReaderConfig) {
    let file_path = get_config_file_path(RPC_STATE_READER_CONFIG);
    let fix = false;
    test_valid_config_body(rpc_state_reader_config, file_path, fix);
}

#[rstest]
#[ignore]
/// Fix the config file for test_valid_rpc_state_reader_config.
/// Run with 'cargo test -- --ignored'.
fn fix_test_valid_rpc_state_reader_config(rpc_state_reader_config: RpcStateReaderConfig) {
    let file_path = get_config_file_path(RPC_STATE_READER_CONFIG);
    let fix = true;
    test_valid_config_body(rpc_state_reader_config, file_path, fix);
}

#[rstest]
/// Read the gateway config and validate its content.
fn test_validate_gateway_config(gateway_config: GatewayConfig) {
    let file_path = get_config_file_path(GATEWAY_CONFIG_FILE);
    let fix = false;
    test_valid_config_body(gateway_config, file_path, fix)
}

#[rstest]
#[ignore]
/// Fix the config file for test_valid_gateway_config.
/// Run with 'cargo test -- --ignored'.
fn fix_test_validate_gateway_config(gateway_config: GatewayConfig) {
    let file_path = get_config_file_path(GATEWAY_CONFIG_FILE);
    let fix = true;
    test_valid_config_body(gateway_config, file_path, fix)
}
