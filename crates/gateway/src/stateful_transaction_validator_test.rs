use blockifier::blockifier::stateful_validator::StatefulValidatorError;
use blockifier::context::BlockContext;
use blockifier::test_utils::CairoVersion;
use blockifier::transaction::errors::{TransactionFeeError, TransactionPreValidationError};
use rstest::rstest;
use starknet_api::felt;
use starknet_api::rpc_transaction::RPCTransaction;
use starknet_api::transaction::TransactionHash;
use starknet_types_core::felt::Felt;
use test_utils::starknet_api_test_utils::{
    declare_tx, deploy_account_tx, invoke_tx, VALID_L1_GAS_MAX_AMOUNT,
    VALID_L1_GAS_MAX_PRICE_PER_UNIT,
};

use crate::compilation::compile_contract_class;
use crate::config::StatefulTransactionValidatorConfig;
use crate::errors::{StatefulTransactionValidatorError, StatefulTransactionValidatorResult};
use crate::stateful_transaction_validator::{
    MockStatefulTransactionValidatorTrait, StatefulTransactionValidator,
};

#[rstest]
#[case::valid_invoke_tx_cairo1(
    invoke_tx(CairoVersion::Cairo1),
    Ok(TransactionHash(felt!(
        "0x007d70505b4487a4e1c1a4b4e4342cb5aa9e73b86d031891170c45a57ad8b4e6"
    )))
)]
#[case::valid_invoke_tx_cairo0(
    invoke_tx(CairoVersion::Cairo0),
    Ok(TransactionHash(felt!(
        "0x032e3a969a64027f15ce2b526d8dff47d47524c58ff0363f93ce4cbe7c280861"
    )))
)]
#[case::valid_deploy_account_tx(
    deploy_account_tx(),
    Ok(TransactionHash(felt!(
        "0x013287740b37dc112391de4ef0f7cd7aeca323537ca2a78a1108c6aee5a55d70"
    )))
)]
#[case::valid_declare_tx(
    declare_tx(),
    Ok(TransactionHash(felt!(
        "0x02da54b89e00d2e201f8e3ed2bcc715a69e89aefdce88aff2d2facb8dec55c0a"
    )))
)]
#[case::invalid_tx(
    invoke_tx(CairoVersion::Cairo1),
    Err(StatefulTransactionValidatorError::StatefulValidatorError(
        StatefulValidatorError::TransactionPreValidationError(
            TransactionPreValidationError::TransactionFeeError(
                TransactionFeeError::L1GasBoundsExceedBalance {
                    max_amount: VALID_L1_GAS_MAX_AMOUNT,
                    max_price: VALID_L1_GAS_MAX_PRICE_PER_UNIT,
                    balance_low: Felt::ZERO,
                    balance_high: Felt::ZERO,
                }
            )
        )
    ))
)]
fn test_stateful_tx_validator(
    #[case] external_tx: RPCTransaction,
    #[case] expected_result: StatefulTransactionValidatorResult<TransactionHash>,
) {
    let block_context = &BlockContext::create_for_testing();
    let stateful_validator = StatefulTransactionValidator {
        config: StatefulTransactionValidatorConfig {
            max_nonce_for_validation_skip: Default::default(),
            validate_max_n_steps: block_context.versioned_constants().validate_max_n_steps,
            max_recursion_depth: block_context.versioned_constants().max_recursion_depth,
            chain_info: block_context.chain_info().clone().into(),
        },
    };
    let optional_class_info = match &external_tx {
        RPCTransaction::Declare(declare_tx) => Some(compile_contract_class(declare_tx).unwrap()),
        _ => None,
    };

    let expected_result_msg = format!("{:?}", expected_result);

    let mut mock_validator = MockStatefulTransactionValidatorTrait::new();
    mock_validator.expect_validate().return_once(|_, _| match expected_result {
        Ok(..) => Ok(()),
        Err(e) => Err(e),
    });

    let result =
        stateful_validator.run_validate(&external_tx, optional_class_info, None, mock_validator);
    assert_eq!(format!("{:?}", result), expected_result_msg);
}
