use std::collections::BTreeMap;

use papyrus_config::dumping::{ser_param, SerializeConfig};
use papyrus_config::{ParamPath, ParamPrivacyInput, SerializedParam};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Validate, PartialEq)]
pub struct GatewayCompilerConfig {
    pub max_bytecode_size: usize,
    pub max_raw_class_size: usize,
}

impl SerializeConfig for GatewayCompilerConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::from_iter([
            ser_param(
                "max_bytecode_size",
                &self.max_bytecode_size,
                "The maximum bytecode size allowed for a contract.",
                ParamPrivacyInput::Public,
            ),
            ser_param(
                "max_raw_class_size",
                &self.max_raw_class_size,
                "The maximum raw class size allowed for a contract.",
                ParamPrivacyInput::Public,
            ),
        ])
    }
}
