use blockifier::blockifier::block::BlockInfo;
use blockifier::blockifier::stateful_validator::StatefulValidator;
use blockifier::bouncer::BouncerConfig;
use blockifier::context::BlockContext;
use blockifier::execution::contract_class::ClassInfo;
use blockifier::state::cached_state::CachedState;
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::versioned_constants::VersionedConstants;
#[cfg(test)]
use mockall::automock;
use starknet_api::rpc_transaction::RPCTransaction;
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

type BlockifierStatefulValidator = StatefulValidator<Box<dyn MempoolStateReader>>;

#[cfg_attr(test, automock)]
pub trait StatefulTransactionValidatorTrait {
    fn perform_validations(
        &mut self,
        account_tx: AccountTransaction,
        deploy_account_tx_hash: Option<TransactionHash>,
    ) -> StatefulTransactionValidatorResult<()>;
}

impl StatefulTransactionValidatorTrait for BlockifierStatefulValidator {
    fn perform_validations(
        &mut self,
        account_tx: AccountTransaction,
        deploy_account_tx_hash: Option<TransactionHash>,
    ) -> StatefulTransactionValidatorResult<()> {
        Ok(self.perform_validations(account_tx, deploy_account_tx_hash)?)
    }
}

impl StatefulTransactionValidator {
    pub fn run_validate<TStatefulTransactionValidator: StatefulTransactionValidatorTrait>(
        &self,
        external_tx: &RPCTransaction,
        optional_class_info: Option<ClassInfo>,
        deploy_account_tx_hash: Option<TransactionHash>,
        mut validator: TStatefulTransactionValidator,
    ) -> StatefulTransactionValidatorResult<TransactionHash> {
        let account_tx = external_tx_to_account_tx(
            external_tx,
            optional_class_info,
            &self.config.chain_info.chain_id,
        )?;
        let tx_hash = get_tx_hash(&account_tx);
        validator.perform_validations(account_tx, deploy_account_tx_hash)?;
        Ok(tx_hash)
    }

    pub fn instantiate_validator(
        &self,
        state_reader_factory: &dyn StateReaderFactory,
    ) -> StatefulTransactionValidatorResult<BlockifierStatefulValidator> {
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
        let block_context = BlockContext::new(
            block_info,
            self.config.chain_info.clone().into(),
            versioned_constants,
            BouncerConfig::max(),
        );

        Ok(BlockifierStatefulValidator::create(
            state,
            block_context,
            self.config.max_nonce_for_validation_skip,
        ))
    }
}

pub fn get_latest_block_info(
    state_reader_factory: &dyn StateReaderFactory,
) -> StatefulTransactionValidatorResult<BlockInfo> {
    let state_reader = state_reader_factory.get_state_reader_from_latest_block();
    Ok(state_reader.get_block_info()?)
}
