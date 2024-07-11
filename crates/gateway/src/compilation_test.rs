use assert_matches::assert_matches;
use blockifier::execution::contract_class::ContractClass;
use cairo_lang_starknet_classes::allowed_libfuncs::AllowedLibfuncsError;
use mempool_test_utils::starknet_api_test_utils::{
    compiled_class_hash, contract_class, declare_tx,
};
use rstest::{fixture, rstest};
use starknet_api::core::CompiledClassHash;
use starknet_api::rpc_transaction::{RPCDeclareTransaction, RPCTransaction};
use starknet_sierra_compile::errors::CompilationUtilError;
use starknet_sierra_compile::utils::into_contract_class_for_compilation;

use crate::compilation::{validate_compiled_class_hash, GatewayCompiler};
use crate::errors::GatewayError;

#[fixture]
fn gateway_compiler() -> GatewayCompiler {
    GatewayCompiler { config: Default::default() }
}

#[rstest]
fn test_compile_contract_class_compiled_class_hash_missmatch(gateway_compiler: GatewayCompiler) {
    let casm_contract_class =
        gateway_compiler.compile(into_contract_class_for_compilation(&contract_class())).unwrap();
    let expected_hash_result = compiled_class_hash();
    let supplied_hash = CompiledClassHash::default();

    let result = validate_compiled_class_hash(&casm_contract_class, supplied_hash);
    assert_matches!(
        result.unwrap_err(),
        GatewayError::CompiledClassHashMismatch { supplied, hash_result }
        if supplied == supplied_hash && hash_result == expected_hash_result
    );
}

#[rstest]
fn test_compile_contract_class_bad_sierra(gateway_compiler: GatewayCompiler) {
    // Create a currupted contract class.
    let mut contract_class = contract_class();
    contract_class.sierra_program = contract_class.sierra_program[..100].to_vec();

    let cairo_lang_contract_class = into_contract_class_for_compilation(&contract_class);
    let result = gateway_compiler.compile(cairo_lang_contract_class);
    assert_matches!(
        result.unwrap_err(),
        GatewayError::CompilationError(CompilationUtilError::AllowedLibfuncsError(
            AllowedLibfuncsError::SierraProgramError
        ))
    )
}

#[rstest]
fn test_handle_declare_tx(gateway_compiler: GatewayCompiler) {
    let declare_tx = assert_matches!(
        declare_tx(),
        RPCTransaction::Declare(declare_tx) => declare_tx
    );
    let RPCDeclareTransaction::V3(declare_tx_v3) = &declare_tx;
    let contract_class = &declare_tx_v3.contract_class;

    let class_info = gateway_compiler.handle_declare_tx(&declare_tx).unwrap();
    assert_matches!(class_info.contract_class(), ContractClass::V1(_));
    assert_eq!(class_info.sierra_program_length(), contract_class.sierra_program.len());
    assert_eq!(class_info.abi_length(), contract_class.abi.len());
}
