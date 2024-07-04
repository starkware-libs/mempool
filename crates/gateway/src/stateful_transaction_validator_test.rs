use blockifier::blockifier::stateful_validator::StatefulValidatorError;
use blockifier::context::BlockContext;
use blockifier::test_utils::CairoVersion;
use blockifier::transaction::errors::{TransactionFeeError, TransactionPreValidationError};
use mempool_test_utils::starknet_api_test_utils::{
    invoke_tx, VALID_L1_GAS_MAX_AMOUNT, VALID_L1_GAS_MAX_PRICE_PER_UNIT,
};
use num_bigint::BigUint;
use rstest::rstest;
use starknet_api::felt;
use starknet_api::rpc_transaction::RPCTransaction;
use starknet_api::transaction::TransactionHash;

use crate::compilation::compile_contract_class;
use crate::config::StatefulTransactionValidatorConfig;
use crate::errors::{StatefulTransactionValidatorError, StatefulTransactionValidatorResult};
use crate::state_reader_test_utils::local_test_state_reader_factory;
use crate::stateful_transaction_validator::{
    MockStatefulTransactionValidatorTrait, StatefulTransactionValidator,
};

#[rstest]
#[case::valid_tx(
    invoke_tx(CairoVersion::Cairo1),
    Ok(TransactionHash(felt!(
        "0x007d70505b4487a4e1c1a4b4e4342cb5aa9e73b86d031891170c45a57ad8b4e6"
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
                    balance: BigUint::ZERO,
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
    mock_validator.expect_perform_validations().return_once(|_, _| match expected_result {
        Ok(..) => Ok(()),
        Err(e) => Err(e),
    });

    let result =
        stateful_validator.run_validate(&external_tx, optional_class_info, None, mock_validator);
    assert_eq!(format!("{:?}", result), expected_result_msg);
}

#[test]
fn test_instantiate_validator() {
    let state_reader_factory = local_test_state_reader_factory(CairoVersion::Cairo1, false);
    let block_context = &BlockContext::create_for_testing();
    let stateful_validator = StatefulTransactionValidator {
        config: StatefulTransactionValidatorConfig {
            max_nonce_for_validation_skip: Default::default(),
            validate_max_n_steps: block_context.versioned_constants().validate_max_n_steps,
            max_recursion_depth: block_context.versioned_constants().max_recursion_depth,
            chain_info: block_context.chain_info().clone().into(),
        },
    };
    let blockifier_validator = stateful_validator.instantiate_validator(&state_reader_factory);
    assert!(blockifier_validator.is_ok());
}
