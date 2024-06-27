use std::fs;
use std::path::Path;

use cairo_lang_starknet_classes::contract_class::{ContractClass, ContractEntryPoints};
use cairo_lang_utils::bigint::BigUintAsHex;
use serde::Deserialize;

/// Same as `ContractClass` - but ignores unnecessary fields like `abi` in deserialization.
#[derive(Deserialize)]
struct DeserializedContractClass {
    pub sierra_program: Vec<BigUintAsHex>,
    pub sierra_program_debug_info: Option<cairo_lang_sierra::debug_info::DebugInfo>,
    pub contract_class_version: String,
    pub entry_points_by_type: ContractEntryPoints,
}

pub(crate) fn contract_class_from_file(file: &Path) -> ContractClass {
    let DeserializedContractClass {
        sierra_program,
        sierra_program_debug_info,
        contract_class_version,
        entry_points_by_type,
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

// Ways to corrupt Sierra programs.
pub fn trancate_sierra_program(sierra_program: &mut [BigUintAsHex]) -> Vec<BigUintAsHex> {
    let trancation_ammount = 100_usize;

    sierra_program[..trancation_ammount].to_vec()
}

pub fn flip_bit(sierra_program: &mut [BigUintAsHex]) -> Vec<BigUintAsHex> {
    let modified_felt = 100_usize;
    let fliped_bit = 15;

    let mut value = sierra_program[modified_felt].value.clone();
    value.set_bit(fliped_bit, !value.bit(fliped_bit));
    sierra_program[modified_felt] = BigUintAsHex { value };
    sierra_program.to_vec()
}
