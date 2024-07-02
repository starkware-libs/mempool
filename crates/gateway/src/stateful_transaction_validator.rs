use blockifier::blockifier::block::BlockInfo;
use blockifier::blockifier::stateful_validator::StatefulValidator as BlockifierStatefulValidator;
use blockifier::bouncer::BouncerConfig;
use blockifier::context::BlockContext;
use blockifier::execution::contract_class::ClassInfo;
use blockifier::state::cached_state::CachedState;
use blockifier::state::state_api::StateReader;
use blockifier::versioned_constants::VersionedConstants;
use starknet_api::core::Nonce;
use starknet_api::hash::StarkFelt;
use starknet_api::rpc_transaction::{RPCInvokeTransaction, RPCTransaction};
use starknet_api::transaction::TransactionHash;

use crate::config::StatefulTransactionValidatorConfig;
use crate::errors::{StatefulTransactionValidatorError, StatefulTransactionValidatorResult};
use crate::state_reader::{MempoolStateReader, StateReaderFactory};
use crate::utils::{external_tx_to_account_tx, get_tx_hash};

#[cfg(test)]
#[path = "stateful_transaction_validator_test.rs"]
mod stateful_transaction_validator_test;

pub struct StatefulTransactionValidator {
    pub config: StatefulTransactionValidatorConfig,
}

impl StatefulTransactionValidator {
    pub fn run_validate(
        &self,
        state_reader_factory: &dyn StateReaderFactory,
        external_tx: &RPCTransaction,
        optional_class_info: Option<ClassInfo>,
    ) -> StatefulTransactionValidatorResult<TransactionHash> {
        // TODO(yael 6/5/2024): consider storing the block_info as part of the
        // StatefulTransactionValidator and update it only once a new block is created.
        let latest_block_info = get_latest_block_info(state_reader_factory)?;
        let prev_block_number = latest_block_info.block_number;
        let state_reader = state_reader_factory.get_state_reader(prev_block_number);
        let state = CachedState::new(state_reader);
        let versioned_constants = VersionedConstants::latest_constants_with_overrides(
            self.config.validate_max_n_steps,
            self.config.max_recursion_depth,
        );
        let mut block_info = latest_block_info;
        block_info.block_number = block_info.block_number.next().ok_or(
            StatefulTransactionValidatorError::OutOfRangeBlockNumber {
                block_number: block_info.block_number,
            },
        )?;
        // TODO(yael 21/4/24): create the block context using pre_process_block once we will be
        // able to read the block_hash of 10 blocks ago from papyrus.
        let block_context = BlockContext::new_unchecked(
            &block_info,
            &self.config.chain_info.clone().into(),
            &versioned_constants,
        );

        let mut validator =
            BlockifierStatefulValidator::create(state, block_context, BouncerConfig::max());
        let account_tx = external_tx_to_account_tx(
            external_tx,
            optional_class_info,
            &self.config.chain_info.chain_id,
        )?;
        let tx_hash = get_tx_hash(&account_tx);

        let skip_validate = skip_stateful_validations(
            external_tx,
            &state_reader_factory.get_state_reader(prev_block_number),
            self.config.max_nonce_for_validation_skip,
        )?;
        validator.perform_validations(account_tx, skip_validate)?;
        Ok(tx_hash)
    }
}

// Check if validation of an invoke transaction should be skipped due to deploy_account not being
// proccessed yet. This feature is used to improve UX for users sending deploy_account + invoke at
// once.
fn skip_stateful_validations(
    tx: &RPCTransaction,
    state_reader: &impl StateReader,
    max_nonce_for_validation_skip: Nonce,
) -> StatefulTransactionValidatorResult<bool> {
    if let RPCTransaction::Invoke(RPCInvokeTransaction::V3(tx)) = tx {
        // check if the transaction nonce is bigger than 1, meaning it is post
        // deploy_account, and the account nonce is zero, meaning the account was not deployed yet.
        // The mempool will also verify that the deploy_account transaction exists.
        let account_nonce = state_reader.get_nonce_at(tx.sender_address)?;
        if tx.nonce >= Nonce(StarkFelt::ONE)
            && tx.nonce <= max_nonce_for_validation_skip
            && account_nonce == Nonce(StarkFelt::ZERO)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn get_latest_block_info(
    state_reader_factory: &dyn StateReaderFactory,
) -> StatefulTransactionValidatorResult<BlockInfo> {
    let state_reader = state_reader_factory.get_state_reader_from_latest_block();
    Ok(state_reader.get_block_info()?)
}
