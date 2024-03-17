use rstest::rstest;

use starknet_api::transaction::{
    DeclareTransaction, DeployAccountTransaction, InvokeTransaction, InvokeTransactionV1,
    Transaction, TransactionVersion,
};

use crate::starknet_api_utils::{zero_resource_bounds_mapping, TransactionParametersExt};
use crate::transaction_validator::{
    StarknetApiTransactionError, TransactionValidator, TransactionValidatorConfig,
    TransactionValidatorError, TransactionValidatorResult,
};

const VALIDATOR_CONFIG_FOR_TESTING: TransactionValidatorConfig = TransactionValidatorConfig {
    block_declare_cairo0: false,
    block_declare_cairo1: false,
    min_allowed_tx_version: TransactionVersion::THREE,
    max_allowed_tx_version: TransactionVersion::THREE,
};

#[rstest]
#[case::block_declare_on_cairo_version_0(
    TransactionValidatorConfig {
        block_declare_cairo0: true,
        ..Default::default()
    },
    Transaction::Declare(starknet_api::transaction::DeclareTransaction::V0(
        starknet_api::transaction::DeclareTransactionV0V1 {..Default::default()}
    )),
    Err(TransactionValidatorError::BlockedTransactionVersion(
        TransactionVersion::ZERO,
        "Declare of Cairo 0 is blocked.".to_string()
    ))
)]
#[case::block_declare_on_cairo_version_1(
    TransactionValidatorConfig {
        block_declare_cairo1: true,
        ..Default::default()
    },
    Transaction::Declare(starknet_api::transaction::DeclareTransaction::V2(
        starknet_api::transaction::DeclareTransactionV2 {..Default::default()}
    )),
    Err(TransactionValidatorError::BlockedTransactionVersion(
        TransactionVersion::TWO,
        "Transaction type is temporarily blocked.".to_string()
    ))
)]
#[case::tx_version_below_minimal(
    TransactionValidatorConfig {
        min_allowed_tx_version: TransactionVersion::THREE,
        max_allowed_tx_version: TransactionVersion::THREE,
        ..Default::default()
    },
    Transaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1 {..Default::default()})),
    Err(TransactionValidatorError::InvalidTransactionVersion(
        TransactionVersion::ONE,
        format!{"Minimal supported version is {:?}.", TransactionVersion::THREE}
    ))
)]
#[case::tx_version_above_maximal(
    TransactionValidatorConfig {
        min_allowed_tx_version: TransactionVersion::ZERO,
        max_allowed_tx_version: TransactionVersion::ZERO,
        ..Default::default()
    },
    Transaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1 {..Default::default()})),
    Err(TransactionValidatorError::InvalidTransactionVersion(
        TransactionVersion::ONE,
        format!{"Maximal supported version is {:?}.", TransactionVersion::ZERO}
    ))
)]
#[case::deprecated_deploy_tx(
    TransactionValidatorConfig {
        ..Default::default()
    },
    Transaction::Deploy(starknet_api::transaction::DeployTransaction {..Default::default()}),
    Err(StarknetApiTransactionError::TransactionTypeNotSupported.into())
)]
#[case::unsupported_l1_handler_tx(
    TransactionValidatorConfig {
        ..Default::default()
    },
    Transaction::L1Handler(starknet_api::transaction::L1HandlerTransaction {..Default::default()}),
    Err(StarknetApiTransactionError::TransactionTypeNotSupported.into())
)]
#[case::valid_declare_tx(
    VALIDATOR_CONFIG_FOR_TESTING,
    Transaction::Declare(DeclareTransaction::create_for_testing(zero_resource_bounds_mapping())),
    Ok(())
)]
#[case::valid_deploy_account_tx(
    VALIDATOR_CONFIG_FOR_TESTING,
    Transaction::DeployAccount(DeployAccountTransaction::create_for_testing(zero_resource_bounds_mapping())),
    Ok(())
)]
#[case::valid_invoke_tx(
    VALIDATOR_CONFIG_FOR_TESTING,
    Transaction::Invoke(InvokeTransaction::create_for_testing(zero_resource_bounds_mapping())),
    Ok(())
)]
fn test_transaction_validator(
    #[case] config: TransactionValidatorConfig,
    #[case] tx: Transaction,
    #[case] expected_result: TransactionValidatorResult<()>,
) {
    let tx_validator = TransactionValidator { config };
    let result = tx_validator.validate(tx);

    assert_eq!(result, expected_result);
}
