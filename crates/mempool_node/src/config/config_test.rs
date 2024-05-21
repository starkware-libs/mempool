#![allow(unused_imports)]
use std::env::{self, args};
use std::fs::File;
use std::ops::IndexMut;
use std::path::{Path, PathBuf};

use assert_json_diff::assert_json_eq;
use assert_matches::assert_matches;
use colored::Colorize;
use papyrus_config::dumping::SerializeConfig;
use papyrus_config::loading::load_and_process_config;
use papyrus_config::presentation::get_config_presentation;
use papyrus_config::validators::ParsedValidationErrors;
use papyrus_config::{SerializationType, SerializedContent, SerializedParam};
use starknet_gateway::config::{GatewayNetworkConfig, StatelessTransactionValidatorConfig};
use test_utils::get_absolute_path;
use validator::Validate;

use crate::config::{
    node_command, ComponentConfig, ComponentExecutionConfig, GatewayConfig, MempoolNodeConfig,
    DEFAULT_CONFIG_PATH,
};

const TEST_FILES_FOLDER: &str = "crates/mempool_node/src/test_files";
const CONFIG_FILE: &str = "mempool_node_config.json";

fn get_config_file(file_name: &str) -> Result<MempoolNodeConfig, papyrus_config::ConfigError> {
    let config_file = File::open(Path::new(TEST_FILES_FOLDER).join(file_name)).unwrap();
    load_and_process_config::<MempoolNodeConfig>(config_file, node_command(), vec![])
}

#[test]
fn test_valid_config() {
    env::set_current_dir(get_absolute_path("")).expect("Couldn't set working dir.");

    // Read the valid config file and validate its content.
    let expected_config = MempoolNodeConfig {
        components: ComponentConfig {
            gateway_component: ComponentExecutionConfig { execute: true },
            mempool_component: ComponentExecutionConfig { execute: false },
        },
        gateway_config: GatewayConfig {
            network_config: GatewayNetworkConfig { ip: "0.0.0.0".parse().unwrap(), port: 8080 },
            stateless_transaction_validator_config: StatelessTransactionValidatorConfig {
                validate_non_zero_l1_gas_fee: true,
                validate_non_zero_l2_gas_fee: false,
                max_calldata_length: 10,
                max_signature_length: 2,
            },
        },
    };
    let loaded_config = get_config_file(CONFIG_FILE).unwrap();

    assert!(loaded_config.validate().is_ok());
    assert_eq!(loaded_config, expected_config);
}

#[test]
fn test_components_config() {
    env::set_current_dir(get_absolute_path("")).expect("Couldn't set working dir.");

    // Read the valid config file and check that the validator finds no errors.
    let mut config = get_config_file(CONFIG_FILE).unwrap();
    assert!(config.validate().is_ok());

    // Invalidate the gateway component and check that the validator finds an error.
    config.components.gateway_component.execute = false;

    assert_matches!(config.validate(), Err(e) => {
        let parse_err = ParsedValidationErrors::from(e);
        let mut error_msg = String::new();
        for error in parse_err.0 {
            if error.param_path == "components.__all__" {
                error_msg.push_str(&error.code);
                break;
            }
        }
        assert_eq!(error_msg, "Invalid components configuration.");
    });

    // Validate the mempool component and check that the validator finds no errors.
    config.components.mempool_component.execute = true;
    assert!(config.validate().is_ok());
}

#[test]
fn test_dump_default_config() {
    env::set_current_dir(get_absolute_path("")).expect("Couldn't set working dir.");

    let default_config = MempoolNodeConfig::default();
    let dumped_default_config = default_config.dump();
    insta::assert_json_snapshot!(dumped_default_config);

    assert!(default_config.validate().is_ok());
}

#[test]
fn default_config_file_is_up_to_date() {
    env::set_current_dir(get_absolute_path("")).expect("Couldn't set working dir.");
    let from_default_config_file: serde_json::Value =
        serde_json::from_reader(File::open(DEFAULT_CONFIG_PATH).unwrap()).unwrap();

    // Create a temporary file and dump the default config to it.
    let mut tmp_file_path = env::temp_dir();
    tmp_file_path.push("cfg.json");
    MempoolNodeConfig::default().dump_to_file(&vec![], tmp_file_path.to_str().unwrap()).unwrap();

    // Read the dumped config from the file.
    let from_code: serde_json::Value =
        serde_json::from_reader(File::open(tmp_file_path).unwrap()).unwrap();

    println!(
        "{}",
        "Default config file doesn't match the default NodeConfig implementation. Please update \
         it using the dump_config binary."
            .purple()
            .bold()
    );
    println!("Diffs shown below.");
    assert_json_eq!(from_default_config_file, from_code)
}
