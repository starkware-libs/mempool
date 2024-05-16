use std::collections::BTreeMap;
use std::net::IpAddr;

use papyrus_config::dumping::{append_sub_config_name, ser_param, SerializeConfig};
use papyrus_config::{ParamPath, ParamPrivacyInput, SerializedParam};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, Serialize, Deserialize, Validate, PartialEq)]
pub struct GatewayConfig {
    pub network_config: GatewayNetworkConfig,
    pub stateless_transaction_validator_config: StatelessTransactionValidatorConfig,
}

impl GatewayConfig {
    pub fn create_for_testing() -> Self {
        Self {
            network_config: GatewayNetworkConfig::create_for_testing(),
            stateless_transaction_validator_config:
                StatelessTransactionValidatorConfig::create_for_testing(),
        }
    }
}

impl SerializeConfig for GatewayConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        vec![
            append_sub_config_name(self.network_config.dump(), "network_config"),
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

/// The gateway network connection related configuration.
#[derive(Clone, Debug, Serialize, Deserialize, Validate, PartialEq)]
pub struct GatewayNetworkConfig {
    pub ip: IpAddr,
    pub port: u16,
}

impl GatewayNetworkConfig {
    pub fn create_for_testing() -> Self {
        Self { ip: "0.0.0.0".parse().unwrap(), port: 8080 }
    }
}

impl SerializeConfig for GatewayNetworkConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::from_iter([
            ser_param(
                "ip",
                &self.ip.to_string(),
                "The gateway server ip.",
                ParamPrivacyInput::Public,
            ),
            ser_param("port", &self.port, "The gateway server port.", ParamPrivacyInput::Public),
        ])
    }
}

impl Default for GatewayNetworkConfig {
    fn default() -> Self {
        Self { ip: "0.0.0.0".parse().unwrap(), port: 8080 }
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

impl StatelessTransactionValidatorConfig {
    pub fn create_for_testing() -> Self {
        Self {
            validate_non_zero_l1_gas_fee: true,
            validate_non_zero_l2_gas_fee: false,
            max_calldata_length: 10,
            max_signature_length: 0,
        }
    }
}

impl SerializeConfig for StatelessTransactionValidatorConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::from_iter([
            ser_param(
                "validate_non_zero_l1_gas_fee",
                &self.validate_non_zero_l1_gas_fee,
                "If true, validates that a transaction has non-zero L1 resource bounds.",
                ParamPrivacyInput::Public,
            ),
            ser_param(
                "validate_non_zero_l2_gas_fee",
                &self.validate_non_zero_l2_gas_fee,
                "If true, validates that a transaction has non-zero L2 resource bounds.",
                ParamPrivacyInput::Public,
            ),
            ser_param(
                "max_signature_length",
                &self.max_signature_length,
                "Validates that a transaction has calldata length less than or equal to this \
                 value.",
                ParamPrivacyInput::Public,
            ),
            ser_param(
                "max_calldata_length",
                &self.max_calldata_length,
                "Validates that a transaction has signature length less than or equal to this \
                 value.",
                ParamPrivacyInput::Public,
            ),
        ])
    }
}
