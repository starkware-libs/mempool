use std::path::{Path, PathBuf};
use test_utils::config::test_valid_config_body;

use crate::config::GatewayNetworkConfig;

const TEST_FILES_FOLDER: &str = "./src/json_files_for_testing";
const NETWORK_CONFIG_FILE: &str = "gateway_network_config.json";

fn get_config_file_path(file_name: &str) -> PathBuf {
    Path::new(TEST_FILES_FOLDER).join(file_name)
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
