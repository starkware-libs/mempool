use crate::GatewayConfig;
use clap::Command;
use papyrus_config::loading::load_and_process_config;
use std::fs::File;
use std::path::Path;
use validator::Validate;

const TEST_FILES_FOLDER: &str = "./src/json_files_for_testing";
const VALID_CONFIG_FILE: &str = "gateway_config.json";
const INVALID_FIELD_NAME_CONFIG_FILE: &str = "invalid_field_name_gateway_config.json";
const INVALID_ADDRESS_CONFIG_FILE: &str = "invalid_address_gateway_config.json";

fn get_config_file(file_name: &str) -> Result<GatewayConfig, papyrus_config::ConfigError> {
    let config_file = File::open(Path::new(TEST_FILES_FOLDER).join(file_name)).unwrap();
    load_and_process_config::<GatewayConfig>(config_file, Command::new(""), vec![])
}

#[test]
fn test_valid_config() {
    // Read the valid config file and validate its content.
    let expected_config = GatewayConfig {
        bind_address: String::from("0.0.0.0:8080"),
    };
    let load_config = get_config_file(VALID_CONFIG_FILE).unwrap();

    assert!(load_config.validate().is_ok());
    assert_eq!(load_config.bind_address, expected_config.bind_address);
}

#[test]
fn test_config_with_invalid_field_name() {
    // Read the config file with the invalid field path and validate it.
    match get_config_file(INVALID_FIELD_NAME_CONFIG_FILE) {
        Ok(_) => panic!("Expected an error, but got a config."),
        Err(e) => assert_eq!(e.to_string(), "missing field `bind_address`".to_owned()),
    }
}

#[test]
fn test_config_with_invalid_address() {
    // Read the config file with the invalid bind_address values and validate it
    let load_config = get_config_file(INVALID_ADDRESS_CONFIG_FILE).unwrap();
    match load_config.validate() {
        Ok(_) => panic!("Expected an error, but got a config."),
        Err(e) => assert_eq!(
            e.field_errors()["bind_address"][0].code,
            "Invalid Socket address.".to_owned()
        ),
    }
}
