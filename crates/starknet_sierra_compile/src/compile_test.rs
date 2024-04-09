use crate::compile::compile_sierra_to_casm;
use crate::test_utils::contract_class_from_file;

#[test]
fn test_compile_sierra_to_casm() {
    let sierra_path = "tests/fixtures/account_faulty.sierra.json";
    let expected_casm_contract_length = 72304;

    let contract_class = contract_class_from_file(sierra_path);
    let casm = compile_sierra_to_casm(contract_class);
    assert_eq!(casm.len(), expected_casm_contract_length);
}
