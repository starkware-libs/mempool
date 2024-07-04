use std::env;
use std::path::{Path, PathBuf};

pub mod starknet_api_test_utils;
use serde::de;
use starknet_api::rpc_transaction::ContractClass;

pub const TEST_FILES_FOLDER: &str = "crates/test_utils/test_files";
pub const CONTRACT_CLASS_FILE: &str = "contract_class.json";
pub const COMPILED_CLASS_HASH_OF_CONTRACT_CLASS: &str =
    "0x01e4f1248860f32c336f93f2595099aaa4959be515e40b75472709ef5243ae17";
pub const FAULTY_ACCOUNT_CLASS_FILE: &str = "faulty_account.sierra.json";

pub fn contract_class() -> ContractClass {
    let path = PathBuf::new().join(TEST_FILES_FOLDER).join(CONTRACT_CLASS_FILE);
    load_resource(path.to_str().unwrap())
}

/// Returns the absolute path from the project root.
pub fn get_absolute_path(relative_path: &str) -> PathBuf {
    Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("../..").join(relative_path)
}

fn load_resource<T: de::DeserializeOwned>(relative_path: &str) -> T {
    let path = get_absolute_path(relative_path);
    serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap()
}

