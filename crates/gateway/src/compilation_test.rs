use assert_matches::assert_matches;
use blockifier::execution::contract_class::ContractClass;
use cairo_lang_starknet_classes::allowed_libfuncs::AllowedLibfuncsError;
use mempool_test_utils::starknet_api_test_utils::declare_tx;
use rstest::rstest;
use starknet_api::core::CompiledClassHash;
use starknet_api::rpc_transaction::{RPCDeclareTransaction, RPCTransaction};
use starknet_sierra_compile::errors::CompilationUtilError;

use crate::compilation::GatewayCompiler;
use crate::compilation_config::GatewayCompilerConfig;
use crate::errors::GatewayError;

#[test]
fn test_compile_contract_class_compiled_class_hash_missmatch() {
    let mut tx = assert_matches!(
        declare_tx(),
        RPCTransaction::Declare(RPCDeclareTransaction::V3(tx)) => tx
    );
    let expected_hash_result = tx.compiled_class_hash;
    let supplied_hash = CompiledClassHash::default();

    tx.compiled_class_hash = supplied_hash;
    let declare_tx = RPCDeclareTransaction::V3(tx);

    let result = GatewayCompiler {
        config: GatewayCompilerConfig { max_bytecode_size: 4800, max_raw_class_size: 111037 },
    }
    .compile_contract_class(&declare_tx);
    assert_matches!(
        result.unwrap_err(),
        GatewayError::CompiledClassHashMismatch { supplied, hash_result }
        if supplied == supplied_hash && hash_result == expected_hash_result
    );
}

#[rstest]
#[case::bytecode_size(
    GatewayCompilerConfig { max_bytecode_size: 1, max_raw_class_size: usize::MAX},
    GatewayError::CasmBytecodeSizeTooLarge { bytecode_size: 4800, max_bytecode_size: 1 }
)]
#[case::raw_class_size(
    GatewayCompilerConfig { max_bytecode_size: usize::MAX, max_raw_class_size: 1},
    GatewayError::CasmContractClassObjectSizeTooLarge {
        contract_class_object_size: 111037, max_contract_class_object_size: 1
    }
)]
fn test_compile_contract_class_size_validation(
    #[case] sierra_to_casm_compilation_config: GatewayCompilerConfig,
    #[case] expected_error: GatewayError,
) {
    let declare_tx = match declare_tx() {
        RPCTransaction::Declare(declare_tx) => declare_tx,
        _ => panic!("Invalid transaction type"),
    };

    let gateway_compiler = GatewayCompiler { config: sierra_to_casm_compilation_config };
    let result = gateway_compiler.compile_contract_class(&declare_tx);
    if let GatewayError::CasmBytecodeSizeTooLarge {
        bytecode_size: expected_bytecode_size, ..
    } = expected_error
    {
        assert_matches!(
            result.unwrap_err(),
            GatewayError::CasmBytecodeSizeTooLarge { bytecode_size, .. }
            if bytecode_size == expected_bytecode_size
        )
    } else if let GatewayError::CasmContractClassObjectSizeTooLarge {
        contract_class_object_size: expected_contract_class_object_size,
        ..
    } = expected_error
    {
        assert_matches!(
            result.unwrap_err(),
            GatewayError::CasmContractClassObjectSizeTooLarge { contract_class_object_size, .. }
            if contract_class_object_size == expected_contract_class_object_size
        )
    }
}

#[test]
fn test_compile_contract_class_bad_sierra() {
    let mut tx = assert_matches!(
        declare_tx(),
        RPCTransaction::Declare(RPCDeclareTransaction::V3(tx)) => tx
    );
    // Truncate the sierra program to trigger an error.
    tx.contract_class.sierra_program = tx.contract_class.sierra_program[..100].to_vec();
    let declare_tx = RPCDeclareTransaction::V3(tx);

    let result = GatewayCompiler { config: Default::default() }.compile_contract_class(&declare_tx);
    assert_matches!(
        result.unwrap_err(),
        GatewayError::CompilationError(CompilationUtilError::AllowedLibfuncsError(
            AllowedLibfuncsError::SierraProgramError
        ))
    )
}

#[test]
fn test_compile_contract_class() {
    let declare_tx = assert_matches!(
        declare_tx(),
        RPCTransaction::Declare(declare_tx) => declare_tx
    );
    let RPCDeclareTransaction::V3(declare_tx_v3) = &declare_tx;
    let contract_class = &declare_tx_v3.contract_class;

    let class_info = GatewayCompiler {
        config: GatewayCompilerConfig { max_bytecode_size: 4800, max_raw_class_size: 111037 },
    }
    .compile_contract_class(&declare_tx)
    .unwrap();
    assert_matches!(class_info.contract_class(), ContractClass::V1(_));
    assert_eq!(class_info.sierra_program_length(), contract_class.sierra_program.len());
    assert_eq!(class_info.abi_length(), contract_class.abi.len());
}
