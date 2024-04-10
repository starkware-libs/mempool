use blockifier::blockifier::block::BlockInfo;
use blockifier::blockifier::stateful_validator::StatefulValidator as BlockifierStatefulValidator;
use blockifier::context::BlockContext;
use blockifier::context::ChainInfo;
use blockifier::execution::contract_class::ClassInfo;
use blockifier::state::cached_state::CachedState;
use blockifier::state::cached_state::GlobalContractCache;
use blockifier::state::state_api::StateReader;
use blockifier::transaction::transaction_execution::Transaction as ExecutionTransaction;
use blockifier::versioned_constants::VersionedConstants;
use starknet_api::core::Nonce;
use starknet_api::transaction::Transaction;
use starknet_api::transaction::TransactionHash;

use crate::errors::TransactionValidatorError;
use crate::errors::TransactionValidatorResult;

pub struct StatefulValidatorConfig {
    pub global_contract_cache_size: usize,
    pub max_nonce_for_validation_skip: Nonce,
    pub chain_info: ChainInfo,
}

pub struct StatefulValidator<S: StateReader> {
    pub config: StatefulValidatorConfig,
    pub state_reader: S,
    pub latest_block_info: BlockInfo,
}

impl<S: StateReader> StatefulValidator<S> {
    pub fn new(config: StatefulValidatorConfig, state_reader: S, block_info: BlockInfo) -> Self {
        // TODO(yael 2/4/24) get block_info from the state.
        Self {
            config,
            state_reader,
            latest_block_info: block_info,
        }
    }

    pub fn run_validate(
        self,
        tx: Transaction,
        deploy_account_tx_hash: Option<TransactionHash>,
        optional_class_info: Option<ClassInfo>,
    ) -> TransactionValidatorResult<()> {
        let mut block_info = self.latest_block_info;

        block_info.block_number = block_info.block_number.next().ok_or(
            TransactionValidatorError::BlockNumberOutOfRange {
                block_number: block_info.block_number,
            },
        )?;

        let block_context = BlockContext::new_unchecked(
            &block_info,
            &self.config.chain_info,
            &VersionedConstants::default(),
        ); // TODO what should be in versioned constants?
        let global_contract_cache =
            GlobalContractCache::new(self.config.global_contract_cache_size);
        let state = CachedState::new(self.state_reader, global_contract_cache);

        let mut validator = BlockifierStatefulValidator::create(
            state,
            block_context,
            self.config.max_nonce_for_validation_skip,
        );

        let tx_hash = TransactionHash::default(); //TODO(yael 28/3/24) - get this from the function that Mohammed is writing.
        if let ExecutionTransaction::AccountTransaction(account_tx) =
            ExecutionTransaction::from_api(tx, tx_hash, optional_class_info, None, None, false)?
        {
            validator.perform_validations(account_tx, deploy_account_tx_hash)?;
            Ok(())
        } else {
            panic!("Only account transactions are accepted by the gateway")
        }
    }
}
