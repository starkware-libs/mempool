use std::path::{Path, PathBuf};
use test_utils::config::test_valid_config_body;

use crate::config::GatewayNetworkConnectionConfig;

const TEST_FILES_FOLDER: &str = "./src/json_files_for_testing";
const CONFIG_FILE: &str = "gateway_network_connection_config.json";

fn get_config_file_path() -> PathBuf {
    Path::new(TEST_FILES_FOLDER).join(CONFIG_FILE)
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
