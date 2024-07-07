/// Deals with config pointers.
use std::sync::OnceLock;

use papyrus_config::dumping::ser_pointer_target_param;
use papyrus_config::{ParamPath, SerializedParam};

type ConfigPointers = Vec<((ParamPath, SerializedParam), Vec<ParamPath>)>;

const MAX_BYTECODE_SIZE: usize = 81_920;
const MAX_RAW_CLASS_SIZE: usize = 4_089_446; // (3.9 * 2_f32.pow(20)) as usize;

// TODO(Arni): Use this to code dedup.
pub fn config_pointers() -> ConfigPointers {
    static CONFIG_POINTERS: OnceLock<ConfigPointers> = OnceLock::new();
    CONFIG_POINTERS
        .get_or_init(|| {
            vec![
                (
                    ser_pointer_target_param(
                        "max_bytecode_size",
                        &MAX_BYTECODE_SIZE,
                        "The maximum bytecode size allowed for a contract.",
                    ),
                    vec![
                        "gateway_config.stateless_tx_validator_config.max_bytecode_size".to_owned(),
                        "gateway_config.gateway_compiler_config.max_bytecode_size".to_owned(),
                    ],
                ),
                (
                    ser_pointer_target_param(
                        "max_raw_class_size",
                        &MAX_RAW_CLASS_SIZE,
                        "The maximum raw class size allowed for a contract.",
                    ),
                    vec![
                        "gateway_config.stateless_tx_validator_config.max_raw_class_size"
                            .to_owned(),
                        "gateway_config.gateway_compiler_config.max_raw_class_size".to_owned(),
                    ],
                ),
            ]
        })
        .to_vec()
}
