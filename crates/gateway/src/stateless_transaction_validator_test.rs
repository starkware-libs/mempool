use rstest::rstest;

use starknet_api::calldata;
use starknet_api::external_transaction::ExternalTransaction;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, Resource, ResourceBounds, ResourceBoundsMapping};

use crate::starknet_api_test_utils::{
    create_external_declare_tx_for_testing, create_external_deploy_account_tx_for_testing,
    create_external_invoke_tx_for_testing, non_zero_l1_resource_bounds_mapping,
    non_zero_l2_resource_bounds_mapping, zero_resource_bounds_mapping,
};
use crate::stateless_transaction_validator::{
    TransactionValidator, TransactionValidatorConfig, TransactionValidatorError,
    TransactionValidatorResult,
};

const VALIDATOR_CONFIG_FOR_TESTING: TransactionValidatorConfig = TransactionValidatorConfig {
    validate_non_zero_l1_gas_fee: true,
    validate_non_zero_l2_gas_fee: false,
    max_calldata_length: 1,
};

#[rstest]
#[case::ignore_resource_bounds(
    TransactionValidatorConfig{
        validate_non_zero_l1_gas_fee: false,
        validate_non_zero_l2_gas_fee: false,
        ..VALIDATOR_CONFIG_FOR_TESTING
    },
    create_external_invoke_tx_for_testing(zero_resource_bounds_mapping(), calldata![]),
    Ok(())
)]
#[case::missing_l1_gas_resource_bounds(
    TransactionValidatorConfig{
        validate_non_zero_l1_gas_fee: true,
        ..VALIDATOR_CONFIG_FOR_TESTING
    },
    create_external_invoke_tx_for_testing(ResourceBoundsMapping::default(), calldata![]),
    Err(TransactionValidatorError::MissingResource { resource: Resource::L1Gas })
)]
#[case::missing_l2_gas_resource_bounds(
    TransactionValidatorConfig{
        validate_non_zero_l1_gas_fee: false,
        validate_non_zero_l2_gas_fee: true,
        ..VALIDATOR_CONFIG_FOR_TESTING
    },
    create_external_invoke_tx_for_testing(ResourceBoundsMapping::default(), calldata![]),
    Err(TransactionValidatorError::MissingResource { resource: Resource::L2Gas })
)]
#[case::zero_l1_gas_resource_bounds(
    TransactionValidatorConfig{
        validate_non_zero_l1_gas_fee: true,
        ..VALIDATOR_CONFIG_FOR_TESTING
    },
    create_external_invoke_tx_for_testing(zero_resource_bounds_mapping(), calldata![]),
    Err(TransactionValidatorError::ZeroFee{
        resource: Resource::L1Gas, resource_bounds: ResourceBounds::default()
    })
)]
#[case::zero_l2_gas_resource_bounds(
    TransactionValidatorConfig{
        validate_non_zero_l1_gas_fee: false,
        validate_non_zero_l2_gas_fee: true,
        ..VALIDATOR_CONFIG_FOR_TESTING
    },
    create_external_invoke_tx_for_testing(non_zero_l1_resource_bounds_mapping(), calldata![]),
    Err(TransactionValidatorError::ZeroFee{
        resource: Resource::L2Gas, resource_bounds: ResourceBounds::default()
    })
)]
#[case::deploy_account_calldata_too_long(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_deploy_account_tx_for_testing(
        non_zero_l1_resource_bounds_mapping(),
        calldata![StarkFelt::from_u128(1),StarkFelt::from_u128(2)],
    ),
    Err(TransactionValidatorError::CalldataTooLong { calldata_length: 2, max_calldata_length: 1 })
)]
#[case::invoke_calldata_too_long(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_invoke_tx_for_testing(
        non_zero_l1_resource_bounds_mapping(),
        calldata![StarkFelt::from_u128(1),StarkFelt::from_u128(2)],
    ),
    Err(TransactionValidatorError::CalldataTooLong { calldata_length: 2, max_calldata_length: 1 })
)]
#[case::valid_declare_tx(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_declare_tx_for_testing(non_zero_l1_resource_bounds_mapping()),
    Ok(())
)]
#[case::valid_deploy_account_tx(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_deploy_account_tx_for_testing(non_zero_l1_resource_bounds_mapping(), calldata![]),
    Ok(())
)]
#[case::valid_invoke_tx(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_invoke_tx_for_testing(non_zero_l1_resource_bounds_mapping(), calldata![]),
    Ok(())
)]
#[case::valid_l2_gas_invoke_tx(
    TransactionValidatorConfig{
        validate_non_zero_l1_gas_fee: false,
        validate_non_zero_l2_gas_fee: true,
        ..VALIDATOR_CONFIG_FOR_TESTING
    },
    create_external_invoke_tx_for_testing(non_zero_l2_resource_bounds_mapping(), calldata![]),
    Ok(())
)]
fn test_transaction_validator(
    #[case] config: TransactionValidatorConfig,
    #[case] tx: ExternalTransaction,
    #[case] expected_result: TransactionValidatorResult<()>,
) {
    let tx_validator = TransactionValidator { config };
    let result = tx_validator.validate(&tx);

    assert_eq!(result, expected_result);
}
