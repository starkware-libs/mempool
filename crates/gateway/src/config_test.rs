use clap::Command;
use papyrus_config::dumping::SerializeConfig;
use papyrus_config::loading::load_and_process_config;
use serde::Deserialize;
use std::fmt::Debug;
use std::fs::File;
use std::path::{Path, PathBuf};
use validator::Validate;

use crate::config::GatewayNetworkConnectionConfig;

const TEST_FILES_FOLDER: &str = "./src/json_files_for_testing";
const CONFIG_FILE: &str = "gateway_network_connection_config.json";

fn get_config_file_path() -> PathBuf {
    Path::new(TEST_FILES_FOLDER).join(CONFIG_FILE)
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

    let config_file = File::open(config_file_path).unwrap();
    let loaded_config =
        load_and_process_config::<T>(config_file, Command::new(""), vec![]).unwrap();

    assert!(loaded_config.validate().is_ok());
    assert_eq!(loaded_config, expected_config);
}

#[test]
/// Read the valid config file and validate its content.
fn test_valid_config() {
    let expected_config = GatewayNetworkConnectionConfig {
        ip: "0.0.0.0".parse().unwrap(),
        port: 8080,
    };
    let file_path = get_config_file_path();
    // To fix the test, set the `fix` parameter to `true`, and run the test.
    let fix = false;
    test_valid_config_body(expected_config, file_path, fix);
}
