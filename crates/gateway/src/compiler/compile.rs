use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::thread;
use thiserror::Error;

// TODO(Arni, 1/05/2024): Remove the dependancy on anyhow. Run in thread.
use anyhow::Context;
use cairo_lang_starknet_classes::allowed_libfuncs::{AllowedLibfuncsError, ListSelector};
use cairo_lang_starknet_classes::casm_contract_class::{
    CasmContractClass, StarknetSierraCompilationError,
};
use cairo_lang_starknet_classes::contract_class::{ContractClass, ContractEntryPoints};
use cairo_lang_utils::bigint::BigUintAsHex;
use serde::Deserialize;

#[cfg(test)]
#[path = "compile_test.rs"]
pub mod compile_test;

struct SierraToCasmCompliationArgs {
    allowed_libfuncs_list_name: Option<String>,
    allowed_libfuncs_list_file: Option<String>,
    add_pythonic_hints: bool,
    max_bytecode_size: usize,
}

/// Same as `ContractClass` - but ignores `abi` in deserialization.
/// Enables loading old contract classes.
#[derive(Deserialize)]
pub struct ContractClassIgnoreAbi {
    pub sierra_program: Vec<BigUintAsHex>,
    pub sierra_program_debug_info: Option<cairo_lang_sierra::debug_info::DebugInfo>,
    pub contract_class_version: String,
    pub entry_points_by_type: ContractEntryPoints,
    pub _abi: Option<serde_json::Value>,
}

#[derive(Debug, Error)]
enum CompilationUtilError {
    #[error("Both allowed libfuncs list name and file were supplied.")]
    AllowedLibfuncsListSource,
    #[error("Failed to read sierra file {file}.")]
    ReadInputError { file: String },
    #[error("Deserialization failed.")]
    DeserializationError,
    #[error(transparent)]
    AllowedLibfuncsError(#[from] AllowedLibfuncsError),
    #[error(transparent)]
    StarknetSierraCompilationError(#[from] StarknetSierraCompilationError),
}

fn starknet_sierra_compile(
    compilation_args: SierraToCasmCompliationArgs,
    file: &str,
) -> Result<CasmContractClass, CompilationUtilError> {
    let SierraToCasmCompliationArgs {
        allowed_libfuncs_list_name,
        allowed_libfuncs_list_file,
        add_pythonic_hints,
        max_bytecode_size,
    } = compilation_args;
    let list_selector = ListSelector::new(allowed_libfuncs_list_name, allowed_libfuncs_list_file)
        .ok_or(CompilationUtilError::AllowedLibfuncsListSource)?;
    let ContractClassIgnoreAbi {
        sierra_program,
        sierra_program_debug_info,
        contract_class_version,
        entry_points_by_type,
        _abi,
    } = serde_json::from_str(
        &fs::read_to_string(file)
            .map_err(|_| CompilationUtilError::ReadInputError { file: file.into() })?,
    )
    .map_err(|_| CompilationUtilError::DeserializationError)?;
    let contract_class = ContractClass {
        sierra_program,
        sierra_program_debug_info,
        contract_class_version,
        entry_points_by_type,
        abi: None,
    };
    contract_class.validate_version_compatible(list_selector)?;

    Ok(CasmContractClass::from_contract_class(
        contract_class,
        add_pythonic_hints,
        max_bytecode_size,
    )?)
}

// TODO(Arni, 1/05/2024): Add the configurable parameters to the function.
pub fn compile_sierra_to_casm(sierra_path: &str) -> Vec<u8> {
    // TODO: Add configurable parameters to the function.
    let compilation_args = SierraToCasmCompliationArgs {
        allowed_libfuncs_list_name: None,
        allowed_libfuncs_list_file: None,
        add_pythonic_hints: true,
        max_bytecode_size: 1000000,
    };

    let sierra_path_clone = sierra_path.to_string(); // Clone sierra_path
    let handle = thread::spawn(move || {
        catch_unwind(AssertUnwindSafe(move || {
            starknet_sierra_compile(compilation_args, &sierra_path_clone)
        }))
    });

    let result = handle.join().expect("Failed to join thread");

    match result {
        Err(e) => {
            // A panic here might be a feature.
            panic!("Compilation panicked: {:?}", e)
        }
        Ok(Err(compilation_util_error)) => match compilation_util_error {
            CompilationUtilError::DeserializationError => {
                unimplemented!("The user provids a badly formatted Sierra file.")
            }
            CompilationUtilError::AllowedLibfuncsError(_) => {
                unimplemented!("The user writes a contract using libfuncs that are not allowed.")
            }
            CompilationUtilError::StarknetSierraCompilationError(_) => {
                unimplemented!("The user writes a contract that fails to compile to Starknet casm")
            }
            _ => {
                // A panic here is a bug.
                panic!("Compilation failed: {:?}", compilation_util_error)
            }
        },
        Ok(Ok(casm_contract)) => serde_json::to_string_pretty(&casm_contract)
            .with_context(|| "Casm contract Serialization failed.")
            .unwrap()
            .into_bytes(),
    }
}
