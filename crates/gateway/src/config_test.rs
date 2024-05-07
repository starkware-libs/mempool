use clap::Command;
use papyrus_config::dumping::SerializeConfig;
use papyrus_config::loading::load_and_process_config;
use serde::Deserialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use validator::Validate;

use crate::config::GatewayNetworkConfig;

const TEST_FILES_FOLDER: &str = "./src/json_files_for_testing";
const CONFIG_FILE: &str = "gateway_network_connection_config.json";

fn get_config_file_path() -> PathBuf {
    Path::new(TEST_FILES_FOLDER).join(CONFIG_FILE)
}

fn get_config_from_file<T: for<'a> Deserialize<'a>>(
    file_path: PathBuf,
) -> Result<T, papyrus_config::ConfigError> {
    let config_file = File::open(file_path).unwrap();
    load_and_process_config(config_file, Command::new(""), vec![])
}

/// Read the valid config file and validate its content.
fn test_valid_config_body(fix: bool) {
    let expected_config = GatewayNetworkConfig {
        ip: "0.0.0.0".parse().unwrap(),
        port: 8080,
    };

    let file_path = get_config_file_path();
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
    // To fix the test, set the `fix` parameter to `true`, and run the test.
    test_valid_config_body(false);
}
