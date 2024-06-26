use std::collections::BTreeMap;

use papyrus_config::dumping::{ser_param, SerializeConfig};
use papyrus_config::{ParamPath, ParamPrivacyInput, SerializedParam};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, Serialize, Deserialize, Validate, PartialEq)]
pub struct GatewayCompilerConfig {
    pub max_bytecode_size: usize,
    pub max_raw_class_size: usize,
}

impl Default for GatewayCompilerConfig {
    fn default() -> Self {
        Self { max_bytecode_size: 81920, max_raw_class_size: 4089446 }
    }
}

impl SerializeConfig for GatewayCompilerConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::from_iter([
            ser_param(
                "max_bytecode_size",
                &self.max_bytecode_size,
                "Limitation of contract bytecode size",
                ParamPrivacyInput::Public,
            ),
            ser_param(
                "max_raw_class_size",
                &self.max_raw_class_size,
                "Limitation of contract class object size",
                ParamPrivacyInput::Public,
            ),
        ])
    }
}
