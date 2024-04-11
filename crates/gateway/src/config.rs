use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::stateless_transaction_validator::StatelessTransactionValidatorConfig;
use papyrus_config::dumping::{append_sub_config_name, ser_param, SerializeConfig};
use papyrus_config::{ParamPath, ParamPrivacyInput, SerializedParam};
use std::net::IpAddr;
use validator::Validate;

/// The gateway configuration.
#[derive(Clone, Debug, Serialize, Deserialize, Validate, PartialEq)]
pub struct GatewayConfig {
    pub ip: IpAddr,
    pub port: u16,

    pub stateless_transaction_validator_config: StatelessTransactionValidatorConfig,
}

impl SerializeConfig for GatewayConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        vec![
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
            ]),
            append_sub_config_name(
                self.stateless_transaction_validator_config.dump(),
                "stateless_transaction_validator_config",
            ),
        ]
        .into_iter()
        .flatten()
        .collect()
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
