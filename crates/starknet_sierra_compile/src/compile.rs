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

pub struct SierraToCasmCompliationArgs {
    allowed_libfuncs_list_name: Option<String>,
    allowed_libfuncs_list_file: Option<String>,
    add_pythonic_hints: bool,
    max_bytecode_size: usize,
}

#[derive(Debug, Error)]
pub enum CompilationUtilError {
    #[error(transparent)]
    AllowedLibfuncsError(#[from] AllowedLibfuncsError),
    #[error("Both allowed libfuncs list name and file were supplied.")]
    AllowedLibfuncsListSource,
    #[error(transparent)]
    JoinError(#[from] JoinError),
    #[error(transparent)]
    StarknetSierraCompilationError(#[from] StarknetSierraCompilationError),
}

pub async fn compile_sierra_to_casm(
    contract_class: ContractClass,
) -> Result<CasmContractClass, CompilationUtilError> {
    let compilation_args = SierraToCasmCompliationArgs {
        allowed_libfuncs_list_name: None,
        allowed_libfuncs_list_file: None,
        add_pythonic_hints: true,
        max_bytecode_size: 1000000,
    };

    // TODO(task_executor).
    tokio::task::spawn_blocking(move || starknet_sierra_compile(compilation_args, contract_class))
        .await?
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
