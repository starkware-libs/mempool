use std::collections::BTreeMap;

use papyrus_config::dumping::SerializeConfig;
use papyrus_config::{ParamPath, SerializedParam};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Validate, PartialEq)]
pub struct GatewayCompilerConfig {}

impl SerializeConfig for GatewayCompilerConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::new()
    }
}
