use std::fs;

use cairo_lang_starknet_classes::contract_class::{ContractClass, ContractEntryPoints};
use cairo_lang_utils::bigint::BigUintAsHex;
use serde::Deserialize;

/// See: https://github.com/starkware-libs/cairo/blob/d0c0f175a8855242d8c6265c55d3f97f8dfdce40/crates/bin/starknet-sierra-compile/src/main.rs#L34-L43
/// Same as `ContractClass` - but ignores `abi` in deserialization.
/// Enables loading old contract classes.
#[derive(Deserialize)]
struct ContractClassIgnoreAbi {
    pub sierra_program: Vec<BigUintAsHex>,
    pub sierra_program_debug_info: Option<cairo_lang_sierra::debug_info::DebugInfo>,
    pub contract_class_version: String,
    pub entry_points_by_type: ContractEntryPoints,
    pub _abi: Option<serde_json::Value>,
}

pub(crate) fn contract_class_from_file(file: &str) -> ContractClass {
    let ContractClassIgnoreAbi {
        sierra_program,
        sierra_program_debug_info,
        contract_class_version,
        entry_points_by_type,
        _abi,
    } = serde_json::from_str(&fs::read_to_string(file).expect("Failed to read input file."))
        .expect("deserialization Failed.");

    ContractClass {
        sierra_program,
        sierra_program_debug_info,
        contract_class_version,
        entry_points_by_type,
        abi: None,
    }
}
