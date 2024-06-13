use blockifier::blockifier::stateful_validator::StatefulValidatorError;
use blockifier::context::BlockContext;
use blockifier::transaction::errors::{TransactionFeeError, TransactionPreValidationError};
use rstest::rstest;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::TransactionHash;

use crate::config::StatefulTransactionValidatorConfig;
use crate::errors::{StatefulTransactionValidatorError, StatefulTransactionValidatorResult};
use crate::starknet_api_test_utils::{
    test_deploy_account_tx_params, test_invoke_tx_params, TestTxParams, VALID_L1_GAS_MAX_AMOUNT,
    VALID_L1_GAS_MAX_PRICE_PER_UNIT,
};
use crate::stateful_transaction_validator::StatefulTransactionValidator;

#[rstest]
#[case::valid_invoke_tx(
    test_invoke_tx_params(false),
    Ok(TransactionHash(StarkFelt::try_from(
        "0x07459d76bd7adec02c25cf7ab0dcb95e9197101d4ada41cae6b465fcb78c0e47"
    ).unwrap()))
)]
#[case::valid_deploy_account_tx(
    test_deploy_account_tx_params(),
    Ok(TransactionHash(StarkFelt::try_from(
        "0x07fb8387575c7f4daa5996a3bb4a3010f4f4af1009b393c73198b8bc5e788c8f"
    ).unwrap()))
)]
#[case::invalid_tx(
    test_invoke_tx_params(true),
    Err(StatefulTransactionValidatorError::StatefulValidatorError(
        StatefulValidatorError::TransactionPreValidationError(
            TransactionPreValidationError::TransactionFeeError(
                TransactionFeeError::L1GasBoundsExceedBalance {
                    max_amount: VALID_L1_GAS_MAX_AMOUNT,
                    max_price: VALID_L1_GAS_MAX_PRICE_PER_UNIT,
                    balance_low: StarkFelt::ZERO,
                    balance_high: StarkFelt::ZERO,
                }
            )
        )
    ))
)]
fn test_stateful_tx_validator(
    #[case] test_tx_params: TestTxParams,
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

    let result = stateful_validator.run_validate(
        &test_tx_params.state_reader_factory,
        &test_tx_params.external_tx,
        None,
        None,
    );
    assert_eq!(format!("{:?}", result), format!("{:?}", expected_result));
}
