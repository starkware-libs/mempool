use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use papyrus_config::dumping::{ser_param, SerializeConfig};
use papyrus_config::{ParamPath, ParamPrivacyInput, SerializedParam};
use std::net::IpAddr;
use validator::Validate;

/// The gateway network related configuration.
#[derive(Clone, Debug, Serialize, Deserialize, Validate, PartialEq)]
pub struct GatewayNetworkConnectionConfig {
    pub ip: IpAddr,
    pub port: u16,
}

impl SerializeConfig for GatewayNetworkConnectionConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::from_iter([
            ser_param(
                "ip",
                &self.ip.to_string(),
                "The gateway server ip.",
                ParamPrivacyInput::Public,
            ),
            ser_param(
                "port",
                &self.port,
                "The gateway server port.",
                ParamPrivacyInput::Public,
            ),
        ])
    }
}

impl Default for GatewayNetworkConnectionConfig {
    fn default() -> Self {
        Self {
            ip: "0.0.0.0".parse().unwrap(),
            port: 8080,
        }
    }
}
