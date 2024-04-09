use crate::compiler::compile::compile_sierra_to_casm;

#[test]
fn test_compile_sierra_to_casm() {
    let sierra_path = "test/fixtures/compiler/account_faulty.sierra.json";
    let casm = compile_sierra_to_casm(sierra_path);
    assert_eq!(casm.len(), 72304);
}
