use cairo_lang_starknet_classes::allowed_libfuncs::ListSelector;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use cairo_lang_starknet_classes::contract_class::ContractClass;

use crate::errors::CompilationUtilError;

#[cfg(test)]
#[path = "compile_test.rs"]
pub mod compile_test;
struct SierraToCasmCompilationArgs {
    list_selector: ListSelector,
    add_pythonic_hints: bool,
}

impl Default for SierraToCasmCompilationArgs {
    fn default() -> Self {
        Self { list_selector: ListSelector::DefaultList, add_pythonic_hints: true }
    }
}

/// This function may panic.
pub fn compile_sierra_to_casm(
    contract_class: ContractClass,
    max_bytecode_size: usize,
) -> Result<CasmContractClass, CompilationUtilError> {
    let compilation_args = SierraToCasmCompilationArgs::default();

    contract_class.validate_version_compatible(compilation_args.list_selector)?;

    Ok(CasmContractClass::from_contract_class(
        contract_class,
        compilation_args.add_pythonic_hints,
        max_bytecode_size,
    )?)
}
