use rstest::rstest;

use starknet_api::calldata;
use starknet_api::external_transaction::ExternalTransaction;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Calldata, Resource, ResourceBounds, TransactionSignature};

use crate::starknet_api_utils::{
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
    max_calldata_length: 1,
    max_signature_length: 1,
};

#[rstest]
#[case::invalid_resource_bounds(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_invoke_tx_for_testing(
        zero_resource_bounds_mapping(), calldata![], TransactionSignature::default()
    ),
    Err(TransactionValidatorError::ZeroFee{
        resource: Resource::L1Gas, resource_bounds: ResourceBounds::default()
    })
)]
#[case::deploy_account_calldata_too_long(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_deploy_account_tx_for_testing(
        non_zero_resource_bounds_mapping(),
        calldata![StarkFelt::from_u128(1),StarkFelt::from_u128(2)],
        TransactionSignature::default()
    ),
    Err(TransactionValidatorError::CalldataTooLong { calldata_length: 2, max_calldata_length: 1 })
)]
#[case::invoke_calldata_too_long(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_invoke_tx_for_testing(
        non_zero_resource_bounds_mapping(),
        calldata![StarkFelt::from_u128(1),StarkFelt::from_u128(2)],
        TransactionSignature::default()
    ),
    Err(TransactionValidatorError::CalldataTooLong { calldata_length: 2, max_calldata_length: 1 })
)]
#[case::declare_signature_too_long(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_declare_tx_for_testing(
        non_zero_resource_bounds_mapping(),
        TransactionSignature(vec![StarkFelt::from_u128(1),StarkFelt::from_u128(2)]),
    ),
    Err(TransactionValidatorError::SignatureTooLong { signature_length: 2, max_signature_length: 1 })

)]
#[case::deploy_account_signature_too_long(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_deploy_account_tx_for_testing(
        non_zero_resource_bounds_mapping(),
        calldata![],
        TransactionSignature(vec![StarkFelt::from_u128(1),StarkFelt::from_u128(2)])
    ),
    Err(TransactionValidatorError::SignatureTooLong { signature_length: 2, max_signature_length: 1 })
)]
#[case::invoke_signature_too_long(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_invoke_tx_for_testing(
        non_zero_resource_bounds_mapping(),
        calldata![],
        TransactionSignature(vec![StarkFelt::from_u128(1),StarkFelt::from_u128(2)])
    ),
    Err(TransactionValidatorError::SignatureTooLong { signature_length: 2, max_signature_length: 1 })
)]
#[case::valid_declare_tx(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_declare_tx_for_testing(
        non_zero_resource_bounds_mapping(), TransactionSignature::default()
    ),
    Ok(())
)]
#[case::valid_deploy_account_tx(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_deploy_account_tx_for_testing(
        non_zero_resource_bounds_mapping(),
        calldata![],
        TransactionSignature::default()
    ),
    Ok(())
)]
#[case::valid_invoke_tx(
    VALIDATOR_CONFIG_FOR_TESTING,
    create_external_invoke_tx_for_testing(
        non_zero_resource_bounds_mapping(),
        calldata![],
        TransactionSignature::default()
    ),
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
