use rstest::{fixture, rstest};
use validator::Validate;

use crate::config::{
    GatewayConfig, GatewayNetworkConfig, RpcStateReaderConfig, StatelessTransactionValidatorConfig,
};

#[fixture]
fn gateway_network_config() -> GatewayNetworkConfig {
    GatewayNetworkConfig { ip: "0.0.0.0".parse().unwrap(), port: 8080 }
}

#[fixture]
fn stateless_transaction_validator_config() -> StatelessTransactionValidatorConfig {
    StatelessTransactionValidatorConfig {
        validate_non_zero_l1_gas_fee: true,
        validate_non_zero_l2_gas_fee: false,
        max_calldata_length: 10,
        max_signature_length: 0,
    }
}

#[fixture]
fn rpc_state_reader_config() -> RpcStateReaderConfig {
    RpcStateReaderConfig {
        url: "http://localhost:8080".to_string(),
        json_rpc_version: "2.0".to_string(),
    }
}

#[fixture]
fn gateway_config(
    gateway_network_config: GatewayNetworkConfig,
    stateless_transaction_validator_config: StatelessTransactionValidatorConfig,
) -> GatewayConfig {
    GatewayConfig { network_config: gateway_network_config, stateless_transaction_validator_config }
}

#[rstest]
fn test_valid_network_config(gateway_network_config: GatewayNetworkConfig) {
    assert!(gateway_network_config.validate().is_ok());
}

#[rstest]
/// Read the stateless transaction validator config file and validate its content.
fn test_valid_stateless_transaction_validator_config(
    stateless_transaction_validator_config: StatelessTransactionValidatorConfig,
) {
    assert!(stateless_transaction_validator_config.validate().is_ok());
}

#[rstest]
/// Read the rpc state reader config file and validate its content.
fn test_valid_rpc_state_reader_config(rpc_state_reader_config: RpcStateReaderConfig) {
    assert!(rpc_state_reader_config.validate().is_ok());
}

#[rstest]
/// Read the gateway config and validate its content.
fn test_validate_gateway_config(gateway_config: GatewayConfig) {
    assert!(gateway_config.validate().is_ok());
}
