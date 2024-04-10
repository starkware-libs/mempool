use blockifier::blockifier::block::BlockInfo;
use blockifier::blockifier::stateful_validator::StatefulValidator as BlockifierStatefulValidator;
use blockifier::context::BlockContext;
use blockifier::context::ChainInfo;
use blockifier::execution::contract_class::ClassInfo;
use blockifier::state::cached_state::CachedState;
use blockifier::state::cached_state::GlobalContractCache;
use blockifier::state::state_api::StateReader;
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::transactions::DeclareTransaction as BlockifierDeclareTransaction;
use blockifier::transaction::transactions::DeployAccountTransaction as BlockifierDeployAccountTransaction;
use blockifier::transaction::transactions::InvokeTransaction as BlockifierInvokeTransaction;
use blockifier::versioned_constants::VersionedConstants;
use starknet_api::core::calculate_contract_address;
use starknet_api::core::ClassHash;
use starknet_api::core::ContractAddress;
use starknet_api::core::Nonce;
use starknet_api::external_transaction::ExternalDeclareTransaction;
use starknet_api::external_transaction::ExternalDeployAccountTransaction;
use starknet_api::external_transaction::ExternalInvokeTransaction;
use starknet_api::external_transaction::ExternalTransaction;
use starknet_api::transaction::DeclareTransaction;
use starknet_api::transaction::DeclareTransactionV3;
use starknet_api::transaction::DeployAccountTransaction;
use starknet_api::transaction::DeployAccountTransactionV3;
use starknet_api::transaction::InvokeTransaction;
use starknet_api::transaction::InvokeTransactionV3;
use starknet_api::transaction::TransactionHash;

use crate::errors::TransactionValidatorError;
use crate::errors::TransactionValidatorResult;

pub struct StatefulTransactionValidatorConfig {
    pub global_contract_cache_size: usize,
    pub max_nonce_for_validation_skip: Nonce,
    pub validate_max_n_steps: u32,
    pub max_recursion_depth: usize,
    pub chain_info: ChainInfo,
}

pub struct StatefulTransactionValidator<S: StateReader + Clone> {
    pub config: StatefulTransactionValidatorConfig,
    pub state_reader: S,

    pub latest_block_info: BlockInfo,
}

impl<S: StateReader + Clone> StatefulTransactionValidator<S> {
    pub fn new(
        config: StatefulTransactionValidatorConfig,
        state_reader: S,
        latest_block_info: BlockInfo,
    ) -> Self {
        // TODO(yael 2/4/24): get block_info from the state.
        Self {
            config,
            state_reader,
            latest_block_info,
        }
    }

    pub fn run_validate(
        &self,
        external_tx: ExternalTransaction,
        deploy_account_tx_hash: Option<TransactionHash>,
        optional_class_info: Option<ClassInfo>,
    ) -> TransactionValidatorResult<()> {
        let mut block_info = self.latest_block_info.clone();

        block_info.block_number = block_info.block_number.next().ok_or(
            TransactionValidatorError::BlockNumberOutOfRange {
                block_number: block_info.block_number,
            },
        )?;
        let versioned_constants = VersionedConstants::latest_constants_with_overrides(
            self.config.validate_max_n_steps,
            self.config.max_recursion_depth,
        );
        let block_context =
            BlockContext::new_unchecked(&block_info, &self.config.chain_info, &versioned_constants);
        let global_contract_cache =
            GlobalContractCache::new(self.config.global_contract_cache_size);
        let state = CachedState::new(self.state_reader.clone(), global_contract_cache);

        let mut validator = BlockifierStatefulValidator::create(
            state,
            block_context,
            self.config.max_nonce_for_validation_skip,
        );

        let account_tx = self.external_tx_to_account_tx(external_tx, optional_class_info)?;

        validator.perform_validations(account_tx, deploy_account_tx_hash)?;
        Ok(())
    }

    fn external_tx_to_account_tx(
        &self,
        external_tx: ExternalTransaction,
        optional_class_info: Option<ClassInfo>,
    ) -> TransactionValidatorResult<AccountTransaction> {
        let tx_hash = TransactionHash::default(); //TODO(yael 15/4/24): make TransactionHasher public in starknet-api
        match external_tx {
            ExternalTransaction::Declare(ExternalDeclareTransaction::V3(tx)) => {
                let declare_tx = DeclareTransaction::V3(DeclareTransactionV3 {
                    class_hash: ClassHash::default(), //TODO(yael 15/4/24): call the starknet-api function once ready
                    resource_bounds: tx.resource_bounds,
                    tip: tx.tip,
                    signature: tx.signature,
                    nonce: tx.nonce,
                    compiled_class_hash: tx.compiled_class_hash,
                    sender_address: tx.sender_address,
                    nonce_data_availability_mode: tx.nonce_data_availability_mode,
                    fee_data_availability_mode: tx.fee_data_availability_mode,
                    paymaster_data: tx.paymaster_data,
                    account_deployment_data: tx.account_deployment_data,
                });
                let class_info =
                    optional_class_info.expect("declare transaction should contain class info");
                let declare_tx =
                    BlockifierDeclareTransaction::new(declare_tx, tx_hash, class_info)?;
                Ok(AccountTransaction::Declare(declare_tx))
            }
            ExternalTransaction::DeployAccount(ExternalDeployAccountTransaction::V3(tx)) => {
                let deploy_account_tx = DeployAccountTransaction::V3(DeployAccountTransactionV3 {
                    resource_bounds: tx.resource_bounds,
                    tip: tx.tip,
                    signature: tx.signature,
                    nonce: tx.nonce,
                    class_hash: tx.class_hash,
                    contract_address_salt: tx.contract_address_salt,
                    constructor_calldata: tx.constructor_calldata,
                    nonce_data_availability_mode: tx.nonce_data_availability_mode,
                    fee_data_availability_mode: tx.fee_data_availability_mode,
                    paymaster_data: tx.paymaster_data,
                });
                let contract_address = calculate_contract_address(
                    deploy_account_tx.contract_address_salt(),
                    deploy_account_tx.class_hash(),
                    &deploy_account_tx.constructor_calldata(),
                    ContractAddress::default(),
                )?;
                let deploy_account_tx = BlockifierDeployAccountTransaction::new(
                    deploy_account_tx,
                    tx_hash,
                    contract_address,
                );
                Ok(AccountTransaction::DeployAccount(deploy_account_tx))
            }
            ExternalTransaction::Invoke(ExternalInvokeTransaction::V3(tx)) => {
                let invoke_tx = InvokeTransaction::V3(InvokeTransactionV3 {
                    resource_bounds: tx.resource_bounds,
                    tip: tx.tip,
                    signature: tx.signature,
                    nonce: tx.nonce,
                    sender_address: tx.sender_address,
                    calldata: tx.calldata,
                    nonce_data_availability_mode: tx.nonce_data_availability_mode,
                    fee_data_availability_mode: tx.fee_data_availability_mode,
                    paymaster_data: tx.paymaster_data,
                    account_deployment_data: tx.account_deployment_data,
                });
                let invoke_tx = BlockifierInvokeTransaction::new(invoke_tx, tx_hash);
                Ok(AccountTransaction::Invoke(invoke_tx))
            }
        }
    }
}
