use clap::Command;
use papyrus_config::dumping::SerializeConfig;
use papyrus_config::loading::load_and_process_config;
use serde::Deserialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use validator::Validate;

use crate::config::GatewayNetworkConfig;

const TEST_FILES_FOLDER: &str = "./src/json_files_for_testing";
const NETWORK_CONFIG_FILE: &str = "gateway_network_connection_config.json";

fn get_config_file_path(file_name: &str) -> PathBuf {
    Path::new(TEST_FILES_FOLDER).join(file_name)
}

fn get_config_from_file<T: for<'a> Deserialize<'a>>(
    file_path: PathBuf,
) -> Result<T, papyrus_config::ConfigError> {
    let config_file = File::open(file_path).unwrap();
    load_and_process_config(config_file, Command::new(""), vec![])
}

/// Read the valid config file and validate its content.
fn test_valid_network_config_body(fix: bool) {
    let expected_config = GatewayNetworkConfig {
        ip: "0.0.0.0".parse().unwrap(),
        port: 8080,
    };

    let file_path = get_config_file_path(NETWORK_CONFIG_FILE);
    if fix {
        expected_config
            .dump_to_file(&vec![], file_path.to_str().unwrap())
            .unwrap();
    }
    let loaded_config = get_config_from_file::<GatewayNetworkConfig>(file_path).unwrap();

    assert!(loaded_config.validate().is_ok());
    assert_eq!(loaded_config, expected_config);
}

#[test]
fn test_valid_config() {
    // TODO(Arni, 1/7/2024): Create a test fix feature. See:
    // https://users.rust-lang.org/t/how-do-i-run-only-a-specific-file-other-than-main-rs/76213/3
    test_valid_network_config_body(false);
}
