use std::panic::{catch_unwind, AssertUnwindSafe};
use std::thread;
use thiserror::Error;

use cairo_lang_starknet_classes::allowed_libfuncs::{AllowedLibfuncsError, ListSelector};
use cairo_lang_starknet_classes::casm_contract_class::{
    CasmContractClass, StarknetSierraCompilationError,
};
use cairo_lang_starknet_classes::contract_class::ContractClass;

#[cfg(test)]
#[path = "compile_test.rs"]
pub mod compile_test;

struct SierraToCasmCompliationArgs {
    allowed_libfuncs_list_name: Option<String>,
    allowed_libfuncs_list_file: Option<String>,
    add_pythonic_hints: bool,
    max_bytecode_size: usize,
}

#[derive(Debug, Error)]
enum CompilationUtilError {
    #[error(transparent)]
    AllowedLibfuncsError(#[from] AllowedLibfuncsError),
    #[error("Both allowed libfuncs list name and file were supplied.")]
    AllowedLibfuncsListSource,
    #[error(transparent)]
    StarknetSierraCompilationError(#[from] StarknetSierraCompilationError),
}

// TODO(Arni, 1/05/2024): Add the configurable parameters to the function.
pub fn compile_sierra_to_casm(contract_class: ContractClass) -> Vec<u8> {
    // TODO: Add configurable parameters to the function.
    let compilation_args = SierraToCasmCompliationArgs {
        allowed_libfuncs_list_name: None,
        allowed_libfuncs_list_file: None,
        add_pythonic_hints: true,
        max_bytecode_size: 1000000,
    };

    let handle = thread::spawn(move || {
        catch_unwind(AssertUnwindSafe(move || {
            starknet_sierra_compile(compilation_args, contract_class)
        }))
    });

    let result = handle.join().expect("Failed to join thread");

    match result {
        Err(e) => {
            // A panic here might be a feature.
            panic!("Compilation panicked: {:?}", e)
        }
        Ok(Err(compilation_util_error)) => match compilation_util_error {
            CompilationUtilError::AllowedLibfuncsError(_) => {
                todo!("The user writes a contract using libfuncs that are not allowed.")
            }
            CompilationUtilError::StarknetSierraCompilationError(_) => {
                todo!("The user writes a contract that fails to compile to Starknet casm")
            }
            CompilationUtilError::AllowedLibfuncsListSource => {
                // A panic here is a bug.
                panic!("Compilation failed: {:?}", compilation_util_error)
            }
        },
        Ok(Ok(casm_contract)) => serde_json::to_string_pretty(&casm_contract)
            .unwrap()
            .into_bytes(),
    }
}

fn starknet_sierra_compile(
    compilation_args: SierraToCasmCompliationArgs,
    contract_class: ContractClass,
) -> Result<CasmContractClass, CompilationUtilError> {
    let list_selector = ListSelector::new(
        compilation_args.allowed_libfuncs_list_name,
        compilation_args.allowed_libfuncs_list_file,
    )
    .ok_or(CompilationUtilError::AllowedLibfuncsListSource)?;
    contract_class.validate_version_compatible(list_selector)?;

    Ok(CasmContractClass::from_contract_class(
        contract_class,
        compilation_args.add_pythonic_hints,
        compilation_args.max_bytecode_size,
    )?)
}
