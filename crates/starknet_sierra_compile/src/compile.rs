use cairo_lang_starknet_classes::allowed_libfuncs::ListSelector;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use cairo_lang_starknet_classes::contract_class::ContractClass;

use crate::errors::CompilationUtilError;

#[cfg(test)]
#[path = "compile_test.rs"]
pub mod compile_test;

/// This function may panic.
pub fn compile_sierra_to_casm(
    contract_class: ContractClass,
    max_bytecode_size: usize,
) -> Result<CasmContractClass, CompilationUtilError> {
    contract_class.validate_version_compatible(ListSelector::DefaultList)?;

    Ok(CasmContractClass::from_contract_class(contract_class, true, max_bytecode_size)?)
}
