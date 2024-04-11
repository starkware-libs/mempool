use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use papyrus_config::dumping::{ser_param, SerializeConfig};
use papyrus_config::{ParamPath, ParamPrivacyInput, SerializedParam};
use std::net::IpAddr;
use validator::Validate;

use crate::stateless_transaction_validator::StatelessTransactionValidatorConfig;

/// The gateway configuration.
#[derive(Clone, Debug, Serialize, Deserialize, Validate, PartialEq)]
pub struct GatewayConfig {
    pub ip: IpAddr,
    pub port: u16,

    pub stateless_transaction_validator_config: StatelessTransactionValidatorConfig,
}

impl SerializeConfig for GatewayConfig {
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

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            ip: "0.0.0.0".parse().unwrap(),
            port: 8080,
            stateless_transaction_validator_config: Default::default(),
        }
    }
}
