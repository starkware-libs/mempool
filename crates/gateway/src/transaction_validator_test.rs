use rstest::rstest;

use starknet_api::{
    calldata,
    transaction::{
        Calldata, InvokeTransaction, InvokeTransactionV1, InvokeTransactionV3, ResourceBounds,
        ResourceBoundsMapping, Transaction, TransactionVersion,
    },
};

use crate::transaction_validator::{
    TransactionValidator, TransactionValidatorConfig, TransactionValidatorError,
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
    Some(TransactionValidatorError::BlockedTransactionVersion(
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
    Some(TransactionValidatorError::BlockedTransactionVersion(
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
    Some(TransactionValidatorError::InvalidTransactionVersion(
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
    Some(TransactionValidatorError::InvalidTransactionVersion(
        TransactionVersion::ONE,
        format!{"Maximal supported version is {:?}.", TransactionVersion::ZERO}
    ))
)]
#[case::deprecated_deploy_tx(
    TransactionValidatorConfig {
        ..Default::default()
    },
    Transaction::Deploy(starknet_api::transaction::DeployTransaction {..Default::default()}),
    Some(TransactionValidatorError::TransactionTypeNotSupported)
)]
#[case::unsupported_l1_handler_tx(
    TransactionValidatorConfig {
        ..Default::default()
    },
    Transaction::L1Handler(starknet_api::transaction::L1HandlerTransaction {..Default::default()}),
    Some(TransactionValidatorError::TransactionTypeNotSupported)
)]
#[case::invalid_max_fee(
    TransactionValidatorConfig {
        min_allowed_tx_version: TransactionVersion::ZERO,
        max_allowed_tx_version: TransactionVersion::THREE,
        enforce_fee: true,
        ..Default::default()
    },
    Transaction::Invoke(InvokeTransaction::V3(InvokeTransactionV3 {
        resource_bounds:ResourceBoundsMapping::try_from(
            vec![
                (
                    starknet_api::transaction::Resource::L1Gas,
                    ResourceBounds{max_amount: 1, max_price_per_unit: 0 }
                ),
                (
                    starknet_api::transaction::Resource::L2Gas,
                    ResourceBounds{max_amount: 0, max_price_per_unit: 0 }
                )
            ]
        ).unwrap(),
        tip: Default::default(),
        signature: Default::default(),
        nonce: Default::default(),
        sender_address: Default::default(),
        calldata: calldata!(),
        nonce_data_availability_mode: starknet_api::data_availability::DataAvailabilityMode::L1,
        fee_data_availability_mode: starknet_api::data_availability::DataAvailabilityMode::L1,
        paymaster_data: Default::default(),
        account_deployment_data: Default::default()
    })),
    Some(TransactionValidatorError::ZeroFee)
)]
#[case::valid_tx(
    TransactionValidatorConfig {
        min_allowed_tx_version: TransactionVersion::ZERO,
        max_allowed_tx_version: TransactionVersion::THREE,
        enforce_fee: true,
        ..Default::default()
    },
    Transaction::Invoke(InvokeTransaction::V3(InvokeTransactionV3 {
        resource_bounds:ResourceBoundsMapping::try_from(
            vec![
                (
                    starknet_api::transaction::Resource::L1Gas,
                    ResourceBounds{max_amount: 1, max_price_per_unit: 1 }
                ),
                (
                    starknet_api::transaction::Resource::L2Gas,
                    ResourceBounds{max_amount: 0, max_price_per_unit: 0 }
                )
            ]
        ).unwrap(),
        tip: Default::default(),
        signature: Default::default(),
        nonce: Default::default(),
        sender_address: Default::default(),
        calldata: calldata!(),
        nonce_data_availability_mode: starknet_api::data_availability::DataAvailabilityMode::L1,
        fee_data_availability_mode: starknet_api::data_availability::DataAvailabilityMode::L1,
        paymaster_data: Default::default(),
        account_deployment_data: Default::default()
    })),
    None
)]
fn test_transaction_version(
    #[case] config: TransactionValidatorConfig,
    #[case] tx: Transaction,
    #[case] expected_error: Option<TransactionValidatorError>,
) {
    let tx_validator = TransactionValidator { config };
    let result = tx_validator.validate(tx);

    if let Some(expected_error) = expected_error {
        assert_eq!(result.unwrap_err(), expected_error);
    } else {
        assert!(result.is_ok());
    }
}
