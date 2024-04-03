use crate::sierra_to_casm_compiler::compile::{
    compile_sierra_to_casm, run_starknet_sierra_to_casm_help, STARKNET_SIERRA_COMPILE_EXE,
};
use rstest::rstest;

#[test]
fn test_compile_sierra_to_casm() {
    let sierra_path = "src/sierra_to_casm_compiler/account_faulty.sierra.json";
    let casm = compile_sierra_to_casm(sierra_path);
    assert_eq!(casm.len(), 72305);
}

#[rstest]
fn test_run_starknet_sierra_to_casm_help(
    #[values(STARKNET_SIERRA_COMPILE_EXE)] compiler_path: &str,
) {
    run_starknet_sierra_to_casm_help(compiler_path);
}
