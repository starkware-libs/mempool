use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use papyrus_config::dumping::{ser_param, SerializeConfig};
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
            ser_param(
                "stateless_transaction_validator_config",
                &self.stateless_transaction_validator_config,
                "The stateless validator configuration.",
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

#[derive(Clone, Debug, Default, Serialize, Deserialize, Validate, PartialEq)]
pub struct StatelessTransactionValidatorConfig {
    // If true, validates that the resource bounds are not zero.
    pub validate_non_zero_l1_gas_fee: bool,
    pub validate_non_zero_l2_gas_fee: bool,

    pub max_calldata_length: usize,
    pub max_signature_length: usize,
}

impl SerializeConfig for StatelessTransactionValidatorConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::from_iter([
            ser_param(
                "validate_non_zero_l1_gas_fee",
                &self.validate_non_zero_l1_gas_fee,
                "If true, validates transactions have non zero L1 resource bounds.",
                ParamPrivacyInput::Public,
            ),
            ser_param(
                "validate_non_zero_l2_gas_fee",
                &self.validate_non_zero_l2_gas_fee,
                "If true, validates transactions have non zero L2 resource bounds.",
                ParamPrivacyInput::Public,
            ),
            ser_param(
                "max_signature_length",
                &self.max_signature_length,
                "Validates transactions have calldata length less than or equal to this value.",
                ParamPrivacyInput::Public,
            ),
            ser_param(
                "max_calldata_length",
                &self.max_calldata_length,
                "Validates transactions have signature length less than or equal to this value.",
                ParamPrivacyInput::Public,
            ),
        ])
    }
}
