use crate::GatewayConfig;
use clap::Command;
use papyrus_config::loading::load_and_process_config;
use std::fs::File;
use std::path::Path;
use validator::Validate;

const TEST_FILES_FOLDER: &str = "./src/json_files_for_testing";
const DEFAULT_GOOD_CONFIG_FILE: &str = "good_gateway_config.json";
const DEFAULT_BAD_CONFIG_PATH: &str = "bad_gateway_config.json";
const DEFAULT_BAD_ADDRESS_CONFIG_PATH: &str = "bad_gateway_address_config.json";

fn get_config_file(file_name: &str) -> Result<GatewayConfig, papyrus_config::ConfigError> {
    let config_file = File::open(Path::new(TEST_FILES_FOLDER).join(file_name)).unwrap();
    load_and_process_config::<GatewayConfig>(config_file, Command::new(""), vec![])
}

#[test]
fn good_config_test() {
    // Read the good config file and validate it.
    let config = GatewayConfig::default();
    let load_config = get_config_file(DEFAULT_GOOD_CONFIG_FILE).unwrap();

    assert!(load_config.validate().is_ok());
    assert_eq!(load_config.bind_address, config.bind_address);
}

#[test]
fn bad_config_test() {
    // Read the config file with the bad field path and validate it.
    match get_config_file(DEFAULT_BAD_CONFIG_PATH) {
        Ok(_) => panic!("Expected an error, but got a config."),
        Err(e) => assert_eq!(e.to_string(), "missing field `bind_address`".to_owned()),
    }
}

#[test]
fn bad_config_address_test() {
    // Read the config file with the bad bind_address values and validate it
    let load_config = get_config_file(DEFAULT_BAD_ADDRESS_CONFIG_PATH).unwrap();
    match load_config.validate() {
        Ok(_) => panic!("Expected an error, but got a config."),
        Err(e) => assert_eq!(
            e.field_errors()["bind_address"][0].code,
            "Invalid Socket address.".to_owned()
        ),
    }
}
