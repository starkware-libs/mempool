use rstest::rstest;

use starknet_api::external_transaction::ExternalTransaction;
use starknet_api::transaction::{Resource, ResourceBounds};

use crate::starknet_api_test_utils::{
    create_external_declare_tx_for_testing, create_external_deploy_account_tx_for_testing,
    create_external_invoke_tx_for_testing, non_zero_resource_bounds_mapping,
    zero_resource_bounds_mapping,
};
use crate::transaction_validator::{
    TransactionValidator, TransactionValidatorConfig, TransactionValidatorError,
    TransactionValidatorResult,
};

const VALIDATOR_CONFIG_FOR_TESTING: TransactionValidatorConfig = TransactionValidatorConfig {
    fee_resource: Resource::L1Gas,
};

#[rstest]
#[case::invalid_resource_bounds(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_invoke_tx_for_testing(zero_resource_bounds_mapping()),
    Err(TransactionValidatorError::ZeroFee{
        resource: Resource::L1Gas, resource_bounds: ResourceBounds::default()
    })
)]
#[case::valid_declare_tx(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_declare_tx_for_testing(non_zero_resource_bounds_mapping()),
    Ok(())
)]
#[case::valid_deploy_account_tx(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_deploy_account_tx_for_testing(non_zero_resource_bounds_mapping(),),
    Ok(())
)]
#[case::valid_invoke_tx(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_invoke_tx_for_testing(non_zero_resource_bounds_mapping()),
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
