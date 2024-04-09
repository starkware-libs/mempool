use cairo_lang_starknet_classes::allowed_libfuncs::{AllowedLibfuncsError, ListSelector};
use cairo_lang_starknet_classes::casm_contract_class::{
    CasmContractClass, StarknetSierraCompilationError,
};
use cairo_lang_starknet_classes::contract_class::ContractClass;
use thiserror::Error;
use tokio::task::JoinError;

#[cfg(test)]
#[path = "compile_test.rs"]
pub mod compile_test;
pub struct SierraToCasmCompilationArgs {
    list_selector: ListSelector,
    add_pythonic_hints: bool,
    max_bytecode_size: usize,
}

#[derive(Debug, Error)]
pub enum CompilationUtilError {
    #[error(transparent)]
    AllowedLibfuncsError(#[from] AllowedLibfuncsError),
    // The compilation was cancelled or paniced.
    #[error(transparent)]
    JoinError(#[from] JoinError),
    #[error(transparent)]
    StarknetSierraCompilationError(#[from] StarknetSierraCompilationError),
}

pub async fn compile_sierra_to_casm(
    contract_class: ContractClass,
) -> Result<CasmContractClass, CompilationUtilError> {
    let compilation_args = SierraToCasmCompilationArgs {
        list_selector: ListSelector::DefaultList,
        add_pythonic_hints: true,
        max_bytecode_size: 1000000,
    };

    // TODO(task_executor).
    let result = tokio::task::spawn_blocking(move || {
        starknet_sierra_compile(compilation_args, contract_class)
    })
    .await;

    // Converts the JoinError (May arraise from painc) to CompilationUtilError.
    result?
}

/// Compiles a Sierra contract to a Casm contract.
/// This function may panic.
fn starknet_sierra_compile(
    compilation_args: SierraToCasmCompilationArgs,
    contract_class: ContractClass,
) -> Result<CasmContractClass, CompilationUtilError> {
    contract_class.validate_version_compatible(compilation_args.list_selector)?;

    Ok(CasmContractClass::from_contract_class(
        contract_class,
        compilation_args.add_pythonic_hints,
        compilation_args.max_bytecode_size,
    )?)
}
