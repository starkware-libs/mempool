use std::panic;

use blockifier::execution::contract_class::{ClassInfo, ContractClass, ContractClassV1};
use blockifier::execution::execution_utils::felt_to_stark_felt;
use cairo_lang_starknet_classes::casm_contract_class::{
    CasmContractClass, CasmContractEntryPoints,
};
use lazy_static::lazy_static;
use starknet_api::core::CompiledClassHash;
use starknet_api::rpc_transaction::RPCDeclareTransaction;
use starknet_api::transaction::Builtin;
use starknet_sierra_compile::compile::{compile_sierra_to_casm, SierraToCasmCompilationArgs};
use starknet_sierra_compile::errors::CompilationUtilError;
use starknet_sierra_compile::utils::into_contract_class_for_compilation;

use crate::errors::{GatewayError, GatewayResult};
use crate::utils::{is_subsequence, IntoOsOrderEnumIteratorExt};

#[cfg(test)]
#[path = "compilation_test.rs"]
mod compilation_test;

/// Formats the contract class for compilation, compiles it, and returns the compiled contract class
/// wrapped in a [`ClassInfo`].
/// Assumes the contract class is of a Sierra program which is compiled to Casm.
pub fn compile_contract_class(declare_tx: &RPCDeclareTransaction) -> GatewayResult<ClassInfo> {
    let RPCDeclareTransaction::V3(tx) = declare_tx;
    let starknet_api_contract_class = &tx.contract_class;
    let cairo_lang_contract_class =
        into_contract_class_for_compilation(starknet_api_contract_class);

    // Compile Sierra to Casm.
    let catch_unwind_result = panic::catch_unwind(|| {
        compile_sierra_to_casm(
            cairo_lang_contract_class,
            SierraToCasmCompilationArgs { max_bytecode_size: 1_000_000, ..Default::default() },
        )
    });
    let casm_contract_class = match catch_unwind_result {
        Ok(compilation_result) => compilation_result?,
        Err(_) => {
            // TODO(Arni): Log the panic.
            return Err(GatewayError::CompilationError(CompilationUtilError::CompilationPanic));
        }
    };
    validate_casm_class(&casm_contract_class)?;

    let hash_result =
        CompiledClassHash(felt_to_stark_felt(&casm_contract_class.compiled_class_hash()));
    if hash_result != tx.compiled_class_hash {
        return Err(GatewayError::CompiledClassHashMismatch {
            supplied: tx.compiled_class_hash,
            hash_result,
        });
    }

    // Convert Casm contract class to Starknet contract class directly.
    let blockifier_contract_class =
        ContractClass::V1(ContractClassV1::try_from(casm_contract_class)?);
    let class_info = ClassInfo::new(
        &blockifier_contract_class,
        starknet_api_contract_class.sierra_program.len(),
        starknet_api_contract_class.abi.len(),
    )?;
    Ok(class_info)
}

// List of supported builtins.
// This is an explicit function so that it is explicitly desiced which builtins are supported.
// If new builtins are added, they should be added here.
fn is_supported_builtin(builtin: &Builtin) -> bool {
    match builtin {
        Builtin::RangeCheck
        | Builtin::Pedersen
        | Builtin::Poseidon
        | Builtin::EcOp
        | Builtin::Ecdsa
        | Builtin::Bitwise
        | Builtin::SegmentArena => true,
        Builtin::Keccak => false,
    }
}

// TODO(Arni): Add to a config.
lazy_static! {
    static ref SUPPORTED_BUILTINS: Vec<String> = {
        Builtin::os_order_iter()
            .filter(is_supported_builtin)
            .map(|builtin| builtin.name().to_string())
            .collect::<Vec<String>>()
    };
}

// TODO(Arni): Add test.
fn validate_casm_class(contract_class: &CasmContractClass) -> Result<(), GatewayError> {
    let CasmContractEntryPoints { external, l1_handler, constructor } =
        &contract_class.entry_points_by_type;
    let entry_points_iterator = external.iter().chain(l1_handler.iter()).chain(constructor.iter());

    for entry_point in entry_points_iterator {
        let builtins = &entry_point.builtins;
        if !is_subsequence(builtins, &SUPPORTED_BUILTINS) {
            return Err(GatewayError::UnsupportedBuiltins {
                builtins: builtins.clone(),
                supported_builtins: SUPPORTED_BUILTINS.to_vec(),
            });
        }
    }
    Ok(())
}
