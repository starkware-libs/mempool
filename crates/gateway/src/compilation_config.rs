use std::collections::BTreeMap;
use std::sync::OnceLock;

use papyrus_config::dumping::SerializeConfig;
use papyrus_config::{ParamPath, SerializedParam};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, Serialize, Deserialize, Validate, PartialEq)]
pub struct GatewayCompilerConfig {
    pub supported_builtins: Vec<String>,
}

impl Default for GatewayCompilerConfig {
    fn default() -> Self {
        Self { supported_builtins: supported_builtins().clone() }
    }
}

impl SerializeConfig for GatewayCompilerConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        todo!("Impelement GatewayCompilerConfig::dump");
    }
}

// TODO(Arni): Use the Builtin enum from Starknet-api, and explicitly tag each builtin as supported
// or unsupported so that the compiler would alert us on new builtins.
fn supported_builtins() -> &'static Vec<String> {
    static SUPPORTED_BUILTINS: OnceLock<Vec<String>> = OnceLock::new();
    SUPPORTED_BUILTINS.get_or_init(|| {
        // The OS expects this order for the builtins.
        const SUPPORTED_BUILTIN_NAMES: [&str; 7] =
            ["pedersen", "range_check", "ecdsa", "bitwise", "ec_op", "poseidon", "segment_arena"];
        SUPPORTED_BUILTIN_NAMES.iter().map(|builtin| builtin.to_string()).collect::<Vec<String>>()
    })
}
