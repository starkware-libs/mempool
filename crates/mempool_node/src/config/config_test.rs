#![allow(unused_imports)]
use std::env::{self, args};
use std::fs::File;
use std::ops::IndexMut;
use std::path::{Path, PathBuf};

use assert_matches::assert_matches;
use papyrus_config::dumping::SerializeConfig;
use papyrus_config::loading::load_and_process_config;
use papyrus_config::presentation::get_config_presentation;
use papyrus_config::validators::ParsedValidationErrors;
use papyrus_config::{SerializationType, SerializedContent, SerializedParam};
use test_utils::config::{get_config_from_file, test_valid_config_body};
use validator::Validate;

use crate::config::{
    node_command, ComponentConfig, ComponentExecutionConfig, GatewayNetworkConfig,
    MempoolNodeConfig,
};

const TEST_FILES_FOLDER: &str = "./src/test_files";
const CONFIG_FILE: &str = "mempool_node_config.json";

fn get_config_file_path(file_name: &str) -> PathBuf {
    Path::new(TEST_FILES_FOLDER).join(file_name)
}

#[test]
/// Read the valid config file and validate its content.
fn test_valid_config() {
    let expected_config = MempoolNodeConfig {
        components: ComponentConfig {
            gateway_component: ComponentExecutionConfig { execute: true },
            mempool_component: ComponentExecutionConfig { execute: false },
        },
        gateway_config: GatewayNetworkConfig { ip: "0.0.0.0".parse().unwrap(), port: 8080 },
    };
    let config_file_path = get_config_file_path(CONFIG_FILE);
    let fix = false;
    test_valid_config_body(expected_config, config_file_path, fix);
}

#[test]
#[ignore]
/// Fix the config file for test_valid_config. Run with 'cargo test -- --ignored'.
fn fix_test_valid_config() {
    let expected_config = MempoolNodeConfig {
        components: ComponentConfig {
            gateway_component: ComponentExecutionConfig { execute: true },
            mempool_component: ComponentExecutionConfig { execute: false },
        },
        gateway_config: GatewayNetworkConfig { ip: "0.0.0.0".parse().unwrap(), port: 8080 },
    };
    let config_file_path = get_config_file_path(CONFIG_FILE);
    let fix = true;
    test_valid_config_body(expected_config, config_file_path, fix);
}

#[test]
fn test_components_config() {
    // Read the valid config file and check that the validator finds no errors.
    let config_file_path = get_config_file_path(CONFIG_FILE);
    let mut config =
        get_config_from_file::<MempoolNodeConfig>(config_file_path, node_command()).unwrap();
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
