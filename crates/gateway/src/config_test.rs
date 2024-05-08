use clap::Command;
use papyrus_config::dumping::SerializeConfig;
use papyrus_config::loading::load_and_process_config;
use serde::Deserialize;
use std::fmt::Debug;
use std::fs::File;
use std::path::{Path, PathBuf};
use validator::Validate;

use crate::config::GatewayNetworkConfig;

const TEST_FILES_FOLDER: &str = "./src/json_files_for_testing";
const NETWORK_CONFIG_FILE: &str = "gateway_network_config.json";

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
        expected_config
            .dump_to_file(&vec![], config_file_path.to_str().unwrap())
            .unwrap();
    }

    let loaded_config: T = get_config_from_file(config_file_path).unwrap();

    assert!(loaded_config.validate().is_ok());
    assert_eq!(loaded_config, expected_config);
}

#[test]
/// Read the valid config file and validate its content.
fn test_valid_config() {
    let expected_config = GatewayNetworkConfig {
        ip: "0.0.0.0".parse().unwrap(),
        port: 8080,
    };
    let file_path = get_config_file_path(NETWORK_CONFIG_FILE);
    // TODO(Arni, 1/7/2024): Create a test fix feature. See:
    // https://users.rust-lang.org/t/how-do-i-run-only-a-specific-file-other-than-main-rs/76213/3
    let fix = false;
    test_valid_config_body(expected_config, file_path, fix);
}
