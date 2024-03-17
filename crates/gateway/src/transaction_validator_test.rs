use rstest::rstest;

use starknet_api::transaction::{
    InvokeTransaction, InvokeTransactionV1, Transaction, TransactionVersion,
};

use crate::transaction_validator::{
    TransactionValidator, TransactionValidatorConfig, TransactionValidatorError,
    TransactionValidatorResult,
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
    Err(TransactionValidatorError::InvalidTransactionType)
)]
#[case::unsupported_l1_handler_tx(
    TransactionValidatorConfig {
        ..Default::default()
    },
    Transaction::L1Handler(starknet_api::transaction::L1HandlerTransaction {..Default::default()}),
    Err(TransactionValidatorError::InvalidTransactionType)
)]
#[case::valid_tx(
    TransactionValidatorConfig {
        min_allowed_tx_version: TransactionVersion::ZERO,
        max_allowed_tx_version: TransactionVersion::THREE,
        ..Default::default()
    },
    Transaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1 {..Default::default()})),
    Ok(())
)]
fn test_transaction_version(
    #[case] config: TransactionValidatorConfig,
    #[case] tx: Transaction,
    #[case] expected_result: TransactionValidatorResult<()>,
) {
    let tx_validator = TransactionValidator { config };
    let result = tx_validator.validate(tx);

    assert_eq!(result, expected_result);
}
