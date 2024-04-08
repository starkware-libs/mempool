use rstest::rstest;

use starknet_api::external_transaction::ExternalTransaction;
use starknet_api::transaction::{Resource, ResourceBounds, ResourceBoundsMapping};

use crate::starknet_api_test_utils::{
    create_external_declare_tx_for_testing, create_external_deploy_account_tx_for_testing,
    create_external_invoke_tx_for_testing, non_zero_l1_resource_bounds_mapping,
    non_zero_l2_resource_bounds_mapping, zero_resource_bounds_mapping,
};
use crate::transaction_validator::{
    TransactionValidator, TransactionValidatorConfig, TransactionValidatorError,
    TransactionValidatorResult,
};

const VALIDATOR_CONFIG_FOR_TESTING: TransactionValidatorConfig = TransactionValidatorConfig {
    validate_non_zero_l1_gas_fee: true,
    validate_non_zero_l2_gas_fee: false,
};

#[rstest]
#[case::ignore_resource_bounds(
    TransactionValidatorConfig{
        validate_non_zero_l1_gas_fee: false,
        validate_non_zero_l2_gas_fee: false,
    },
    create_external_invoke_tx_for_testing(zero_resource_bounds_mapping()),
    Ok(())
)]
#[case::malformed_resource_bounds(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_invoke_tx_for_testing(ResourceBoundsMapping::default()),
    Err(TransactionValidatorError::MissingResource { resource: Resource::L1Gas })
)]
#[case::malformed_l2_gas_resource_bounds(
    TransactionValidatorConfig{
        validate_non_zero_l1_gas_fee: false,
        validate_non_zero_l2_gas_fee: true,
    },
    create_external_invoke_tx_for_testing(ResourceBoundsMapping::default()),
    Err(TransactionValidatorError::MissingResource { resource: Resource::L2Gas })
)]
#[case::invalid_resource_bounds(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_invoke_tx_for_testing(zero_resource_bounds_mapping()),
    Err(TransactionValidatorError::ZeroFee{
        resource: Resource::L1Gas, resource_bounds: ResourceBounds::default()
    })
)]
#[case::invalid_l2_gas_resource_bounds(
    TransactionValidatorConfig{
        validate_non_zero_l1_gas_fee: false,
        validate_non_zero_l2_gas_fee: true,
    },
    create_external_invoke_tx_for_testing(non_zero_l1_resource_bounds_mapping()),
    Err(TransactionValidatorError::ZeroFee{
        resource: Resource::L2Gas, resource_bounds: ResourceBounds::default()
    })
)]
#[case::valid_declare_tx(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_declare_tx_for_testing(non_zero_l1_resource_bounds_mapping()),
    Ok(())
)]
#[case::valid_deploy_account_tx(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_deploy_account_tx_for_testing(non_zero_l1_resource_bounds_mapping(),),
    Ok(())
)]
#[case::valid_invoke_tx(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_invoke_tx_for_testing(non_zero_l1_resource_bounds_mapping()),
    Ok(())
)]
#[case::valid_l2_gas_invoke_tx(
    TransactionValidatorConfig{
        validate_non_zero_l1_gas_fee: false,
        validate_non_zero_l2_gas_fee: true,
    },
    create_external_invoke_tx_for_testing(non_zero_l2_resource_bounds_mapping()),
    Ok(())
)]
fn test_transaction_validator(
    #[case] config: TransactionValidatorConfig,
    #[case] tx: ExternalTransaction,
    #[case] expected_result: TransactionValidatorResult<()>,
) {
    let tx_validator = TransactionValidator { config };
    let result = tx_validator.validate(tx);

    assert_eq!(result, expected_result);
}
