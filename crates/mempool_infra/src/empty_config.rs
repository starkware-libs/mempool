use std::collections::BTreeMap;

use papyrus_config::dumping::SerializeConfig;
use papyrus_config::{ParamPath, SerializedParam};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EmptyConfig;

impl SerializeConfig for EmptyConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::new()
    }
}
