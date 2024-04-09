use cairo_lang_starknet_classes::allowed_libfuncs::AllowedLibfuncsError;
use cairo_lang_starknet_classes::contract_class::ContractClass;
use std::path::Path;

use crate::compile::{compile_sierra_to_casm, CompilationUtilError};
use crate::test_utils::contract_class_from_file;

const FAULTY_ACCOUNT_SIERRA_FILE: &str = "account_faulty.sierra.json";
const TEST_FILES_FOLDER: &str = "./tests/fixtures";

#[tokio::test]
async fn test_compile_sierra_to_casm() {
    let sierra_path = &Path::new(TEST_FILES_FOLDER).join(FAULTY_ACCOUNT_SIERRA_FILE);
    let expected_casm_contract_length = 72304;

    let contract_class = contract_class_from_file(sierra_path);
    let casm_contract = compile_sierra_to_casm(contract_class).await.unwrap();
    let serialized_casm = serde_json::to_string_pretty(&casm_contract)
        .unwrap()
        .into_bytes();

    assert_eq!(serialized_casm.len(), expected_casm_contract_length);
}

// TODO(Arni, 1/5/2024): Add a test for panic result test.
#[tokio::test]
async fn test_negative_flow_compile_sierra_to_casm() {
    let sierra_path = &Path::new(TEST_FILES_FOLDER).join(FAULTY_ACCOUNT_SIERRA_FILE);

    let contract_class = contract_class_from_file(sierra_path);
    let ContractClass {
        sierra_program,
        sierra_program_debug_info,
        contract_class_version,
        entry_points_by_type,
        abi,
    } = contract_class;

    let faulty_sierra_program = sierra_program[..100].to_vec();
    let faulty_contract_class = ContractClass {
        sierra_program: faulty_sierra_program,
        sierra_program_debug_info,
        contract_class_version,
        entry_points_by_type,
        abi,
    };
    let result = compile_sierra_to_casm(faulty_contract_class).await;
    if let CompilationUtilError::AllowedLibfuncsError(AllowedLibfuncsError::SierraProgramError) =
        result.unwrap_err()
    {
        return;
    } else {
        panic!("Unexpected error.")
    }
}
