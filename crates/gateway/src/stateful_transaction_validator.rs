use blockifier::blockifier::block::BlockInfo;
use blockifier::blockifier::stateful_validator::StatefulValidator as BlockifierStatefulValidator;
use blockifier::bouncer::BouncerConfig;
use blockifier::context::BlockContext;
use blockifier::context::ChainInfo;
use blockifier::execution::contract_class::ClassInfo;
use blockifier::state::cached_state::CachedState;
use blockifier::state::state_api::StateReader;

use blockifier::versioned_constants::VersionedConstants;
use starknet_api::core::Nonce;
use starknet_api::external_transaction::ExternalTransaction;
use starknet_api::transaction::TransactionHash;

use crate::errors::StatefulTransactionValidatorError;
use crate::errors::StatefulTransactionValidatorResult;
use crate::utils::external_tx_to_account_tx;

#[cfg(test)]
#[path = "stateful_transaction_validator_test.rs"]
mod stateful_transaction_validator_test;

pub struct StatefulTransactionValidatorConfig {
    pub max_nonce_for_validation_skip: Nonce,
    pub validate_max_n_steps: u32,
    pub max_recursion_depth: usize,
    pub chain_info: ChainInfo,
}

pub struct StatefulTransactionValidator<S: StateReader> {
    pub latest_block_info: BlockInfo,
    validator: BlockifierStatefulValidator<S>,
}

impl<S: StateReader> StatefulTransactionValidator<S> {
    pub fn create_and_run_validate(
        config: &StatefulTransactionValidatorConfig,
        state_reader: S,
        latest_block_info: BlockInfo,
        external_tx: &ExternalTransaction,
        deploy_account_tx_hash: Option<TransactionHash>,
        optional_class_info: Option<ClassInfo>,
    ) -> StatefulTransactionValidatorResult<()> {
        let mut validator = Self::new(config, state_reader, latest_block_info);
        validator.run_validate(external_tx, deploy_account_tx_hash, optional_class_info)
    }

    pub fn new(
        config: &StatefulTransactionValidatorConfig,
        state_reader: S,
        // TODO(yael 2/4/24): get block_info from the state.
        latest_block_info: BlockInfo,
    ) -> Self {
        let state = CachedState::new(state_reader);
        let versioned_constants = VersionedConstants::latest_constants_with_overrides(
            config.validate_max_n_steps,
            config.max_recursion_depth,
        );
        let block_context = BlockContext::new_unchecked(
            &latest_block_info,
            &config.chain_info,
            &versioned_constants,
        );

        let validator = BlockifierStatefulValidator::create(
            state,
            block_context,
            config.max_nonce_for_validation_skip,
            BouncerConfig::max(),
        );
        Self {
            latest_block_info,
            validator,
        }
    }

    pub fn run_validate(
        &mut self,
        external_tx: &ExternalTransaction,
        deploy_account_tx_hash: Option<TransactionHash>,
        optional_class_info: Option<ClassInfo>,
    ) -> StatefulTransactionValidatorResult<()> {
        let mut block_info = self.latest_block_info.clone();
        block_info.block_number = block_info.block_number.next().ok_or(
            StatefulTransactionValidatorError::BlockNumberOutOfRange {
                block_number: block_info.block_number,
            },
        )?;

        let account_tx = external_tx_to_account_tx(external_tx, optional_class_info)?;
        self.validator
            .perform_validations(account_tx, deploy_account_tx_hash)?;
        Ok(())
    }
}
