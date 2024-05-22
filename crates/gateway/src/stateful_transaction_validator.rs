use blockifier::blockifier::block::BlockInfo;
use blockifier::blockifier::stateful_validator::StatefulValidator as BlockifierStatefulValidator;
use blockifier::bouncer::BouncerConfig;
use blockifier::context::{BlockContext, ChainInfo};
use blockifier::execution::contract_class::ClassInfo;
use blockifier::state::cached_state::CachedState;
use blockifier::versioned_constants::VersionedConstants;
use starknet_api::core::Nonce;
use starknet_api::external_transaction::ExternalTransaction;
use starknet_api::transaction::TransactionHash;

use crate::errors::{StatefulTransactionValidatorError, StatefulTransactionValidatorResult};
use crate::state_reader::{MempoolStateReader, StateReaderFactory};
use crate::utils::external_tx_to_account_tx;

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
        external_tx: &ExternalTransaction,
        optional_class_info: Option<ClassInfo>,
        deploy_account_tx_hash: Option<TransactionHash>,
    ) -> StatefulTransactionValidatorResult<()> {
        // TODO(yael 6/5/2024): consider storing the block_info as part of the
        // StatefulTransactionValidator and update it only once a new block is created.
        let latest_block_info = get_latest_block_info(state_reader_factory)?;
        let state_reader = state_reader_factory.get_state_reader(latest_block_info.block_number);
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
        let block_context =
            BlockContext::new_unchecked(&block_info, &self.config.chain_info, &versioned_constants);

        let mut validator = BlockifierStatefulValidator::create(
            state,
            block_context,
            self.config.max_nonce_for_validation_skip,
            BouncerConfig::max(),
        );
        let account_tx = external_tx_to_account_tx(
            external_tx,
            optional_class_info,
            &self.config.chain_info.chain_id,
        )?;
        validator.perform_validations(account_tx, deploy_account_tx_hash)?;
        Ok(())
    }
}

pub fn get_latest_block_info(
    state_reader_factory: &dyn StateReaderFactory,
) -> StatefulTransactionValidatorResult<BlockInfo> {
    let state_reader = state_reader_factory.get_state_reader_from_latest_block();
    Ok(state_reader.get_block_info()?)
}

pub struct StatefulTransactionValidatorConfig {
    pub max_nonce_for_validation_skip: Nonce,
    pub validate_max_n_steps: u32,
    pub max_recursion_depth: usize,
    pub chain_info: ChainInfo,
}

impl StatefulTransactionValidatorConfig {
    pub fn create_for_testing() -> Self {
        StatefulTransactionValidatorConfig {
            max_nonce_for_validation_skip: Default::default(),
            validate_max_n_steps: 1000000,
            max_recursion_depth: 50,
            chain_info: BlockContext::create_for_testing().chain_info().clone(),
        }
    }
}
