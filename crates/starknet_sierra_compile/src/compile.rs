use cairo_lang_starknet_classes::allowed_libfuncs::ListSelector;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use cairo_lang_starknet_classes::contract_class::ContractClass;

use crate::errors::CompilationUtilError;

#[cfg(test)]
#[path = "compile_test.rs"]
pub mod compile_test;
pub struct SierraToCasmCompilationArgs {
    pub list_selector: ListSelector,
    pub add_pythonic_hints: bool,
    pub max_bytecode_size: usize,
}

impl Default for SierraToCasmCompilationArgs {
    fn default() -> Self {
        Self {
            list_selector: ListSelector::DefaultList,
            add_pythonic_hints: true,
            max_bytecode_size: Default::default(),
        }
    }
}

/// This function may panic.
pub fn compile_sierra_to_casm(
    contract_class: ContractClass,
    compilation_args: SierraToCasmCompilationArgs,
) -> Result<CasmContractClass, CompilationUtilError> {
    contract_class.validate_version_compatible(compilation_args.list_selector)?;

    Ok(CasmContractClass::from_contract_class(
        contract_class,
        compilation_args.add_pythonic_hints,
        compilation_args.max_bytecode_size,
    )?)
}
