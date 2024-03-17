use rstest::rstest;

use starknet_api::transaction::InvokeTransaction;
use starknet_api::transaction::InvokeTransactionV1;
use starknet_api::transaction::Transaction;
use starknet_api::transaction::TransactionVersion;

use crate::errors::GatewayError;
use crate::errors::TransactionValidatorError;
use crate::transaction_validator::{TransactionValidator, TransactionValidatorConfig};

#[rstest]
#[case::block_declare_on_cairo_version_0(
    TransactionValidatorConfig {
        block_declare_cairo0: true,
        ..Default::default()
    },
    Transaction::Declare(starknet_api::transaction::DeclareTransaction::V0(
        starknet_api::transaction::DeclareTransactionV0V1 {..Default::default()}
    )),
    GatewayError::TransactionValidatorError(
        TransactionValidatorError::BlockedTransactionVersion(
            TransactionVersion::ZERO,
            "Declare of Cairo 0 is blocked.".to_string()
        )
    )
)]
#[case::block_declare_on_cairo_version_1(
    TransactionValidatorConfig {
        block_declare_cairo1: true,
        ..Default::default()
    },
    Transaction::Declare(starknet_api::transaction::DeclareTransaction::V2(
        starknet_api::transaction::DeclareTransactionV2 {..Default::default()}
    )),
    GatewayError::TransactionValidatorError(
        TransactionValidatorError::BlockedTransactionVersion(
            TransactionVersion::TWO,
            "Transaction type is temporarily blocked.".to_string()
        )
    )
)]
#[case::tx_version_below_minimal(
    TransactionValidatorConfig {
        min_allowed_tx_version: 3,
        max_allowed_tx_version: 3,
        current_tx_version: 3,
        ..Default::default()
    },
    Transaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1 {..Default::default()})),
    GatewayError::TransactionValidatorError(
        TransactionValidatorError::InvalidTransactionVersion(
            TransactionVersion::ONE,
            "Minimal supported version is 3.".to_string()
        )
    )
)]
#[case::tx_version_above_maximal(
    TransactionValidatorConfig {
        min_allowed_tx_version: 0,
        max_allowed_tx_version: 0,
        current_tx_version: 1,
        ..Default::default()
    },
    Transaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1 {..Default::default()})),
    GatewayError::TransactionValidatorError(
        TransactionValidatorError::InvalidTransactionVersion(
            TransactionVersion::ONE,
            "Maximal supported version is 0.".to_string()
        )
    )
)]
#[case::tx_version_above_current(
    TransactionValidatorConfig {
        min_allowed_tx_version: 0,
        max_allowed_tx_version: 0,
        current_tx_version: 0,
        ..Default::default()
    },
    Transaction::Invoke(InvokeTransaction::V1(InvokeTransactionV1 {..Default::default()})),
    GatewayError::TransactionValidatorError(
        TransactionValidatorError::InvalidTransactionVersion(
            TransactionVersion::ONE,
            "Maximal valid version is 0.".to_string()
        )
    )
)]
#[case::deprecated_deploy_tx(
    TransactionValidatorConfig {
        ..Default::default()
    },
    Transaction::Deploy(starknet_api::transaction::DeployTransaction {..Default::default()}),
    GatewayError::TransactionValidatorError(
        TransactionValidatorError::InvalidTransactionType
    )
)]
#[case::unsupported_l1_handler_tx(
    TransactionValidatorConfig {
        ..Default::default()
    },
    Transaction::L1Handler(starknet_api::transaction::L1HandlerTransaction {..Default::default()}),
    GatewayError::TransactionValidatorError(
        TransactionValidatorError::InvalidTransactionType
    )
)]
fn test_transaction_version(
    #[case] config: TransactionValidatorConfig,
    #[case] tx: Transaction,
    #[case] expected_error: GatewayError,
) {
    let tx_validator = TransactionValidator { config };
    let result = tx_validator.validate(tx);

    assert_eq!(result.unwrap_err(), expected_error);
}
