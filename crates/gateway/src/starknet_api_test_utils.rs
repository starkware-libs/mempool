use std::collections::HashMap;

use blockifier::test_utils::contracts::FeatureContract;
use blockifier::test_utils::{create_trivial_calldata, CairoVersion, NonceManager};
use serde_json::to_string_pretty;
use starknet_api::core::{ClassHash, CompiledClassHash, ContractAddress, Nonce};
use starknet_api::data_availability::DataAvailabilityMode;
use starknet_api::external_transaction::{
    ContractClass, ExternalDeclareTransaction, ExternalDeclareTransactionV3,
    ExternalDeployAccountTransaction, ExternalDeployAccountTransactionV3,
    ExternalInvokeTransaction, ExternalInvokeTransactionV3, ExternalTransaction,
    ResourceBoundsMapping,
};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{
    AccountDeploymentData, Calldata, ContractAddressSalt, PaymasterData, ResourceBounds, Tip,
    TransactionSignature, TransactionVersion,
};
use starknet_api::{calldata, stark_felt};

use crate::{declare_tx_args, deploy_account_tx_args, invoke_tx_args};

pub const VALID_L1_GAS_MAX_AMOUNT: u64 = 2214;
pub const VALID_L1_GAS_MAX_PRICE_PER_UNIT: u128 = 100000000000;

// Utils.
pub enum TransactionType {
    Declare,
    DeployAccount,
    Invoke,
}

pub fn get_sender_address(tx: &ExternalTransaction) -> ContractAddress {
    match tx {
        ExternalTransaction::Declare(ExternalDeclareTransaction::V3(tx)) => tx.sender_address,
        // TODO(Mohammad): Add support for deploy account.
        ExternalTransaction::DeployAccount(ExternalDeployAccountTransaction::V3(_)) => {
            ContractAddress::default()
        }
        ExternalTransaction::Invoke(ExternalInvokeTransaction::V3(tx)) => tx.sender_address,
    }
}

pub fn external_tx_for_testing(
    tx_type: TransactionType,
    resource_bounds: ResourceBoundsMapping,
    calldata: Calldata,
    signature: TransactionSignature,
) -> ExternalTransaction {
    match tx_type {
        TransactionType::Declare => {
            let contract_class = ContractClass {
                sierra_program: vec![stark_felt!(1_u32); 3],
                ..ContractClass::default()
            };
            external_declare_tx(declare_tx_args!(resource_bounds, signature, contract_class))
        }
        TransactionType::DeployAccount => external_deploy_account_tx(
            deploy_account_tx_args!(resource_bounds, constructor_calldata: calldata, signature),
        ),
        TransactionType::Invoke => {
            external_invoke_tx(invoke_tx_args!(signature, resource_bounds, calldata))
        }
    }
}

pub const NON_EMPTY_RESOURCE_BOUNDS: ResourceBounds =
    ResourceBounds { max_amount: 1, max_price_per_unit: 1 };

pub fn create_resource_bounds_mapping(
    l1_resource_bounds: ResourceBounds,
    l2_resource_bounds: ResourceBounds,
) -> ResourceBoundsMapping {
    ResourceBoundsMapping { l1_gas: l1_resource_bounds, l2_gas: l2_resource_bounds }
}

pub fn zero_resource_bounds_mapping() -> ResourceBoundsMapping {
    create_resource_bounds_mapping(ResourceBounds::default(), ResourceBounds::default())
}

pub fn executable_resource_bounds_mapping() -> ResourceBoundsMapping {
    create_resource_bounds_mapping(
        ResourceBounds {
            max_amount: VALID_L1_GAS_MAX_AMOUNT,
            max_price_per_unit: VALID_L1_GAS_MAX_PRICE_PER_UNIT,
        },
        ResourceBounds::default(),
    )
}

// Convenience method for generating a single invoke transaction with trivial fields.
// For multiple, nonce-incrementing transactions, use the transaction generator directly.
pub fn invoke_tx() -> ExternalTransaction {
    MultiAccountTransactionGenerator::new(1).account(0).generate(invoke_tx_args!())
}

type TransactionGeneratorAccountId = u16;

/// Manages transaction generation for multiple pre-funded accounts, internally bumping nonces
/// as needed.
///
/// **Currently supports:**
/// - Single account type
/// - Single contract type
/// - Only supports invoke, which are all a trivial method in the contract type.
///
/// # Example
///
/// ```
/// use starknet_gateway::invoke_tx_args;
/// use starknet_gateway::starknet_api_test_utils::MultiAccountTransactionGenerator;
///
/// let mut tx_generator = MultiAccountTransactionGenerator::new(2); // Initialize with 2 accounts.
/// let account_0_tx_with_nonce_0 = tx_generator.account(0).generate(invoke_tx_args!());
/// let account_1_tx_with_nonce_0 = tx_generator.account(1).generate(invoke_tx_args!());
/// let account_0_tx_with_nonce_1 = tx_generator.account(0).generate(invoke_tx_args!());
/// ```
// Note: when moving this to starknet api crate, see if blockifier's
// [blockifier::transaction::test_utils::FaultyAccountTxCreatorArgs] can be made to use this.
pub struct MultiAccountTransactionGenerator {
    account_contracts: HashMap<TransactionGeneratorAccountId, ContractAddress>,
    // Invariant: nonces managed internally thorugh `generate` API.
    internal_nonce_manager: NonceManager,
}

impl MultiAccountTransactionGenerator {
    pub fn new(n_accounts: TransactionGeneratorAccountId) -> Self {
        let default_account_contract =
            FeatureContract::AccountWithoutValidations(CairoVersion::Cairo1);
        Self::new_for_account_contract(default_account_contract, n_accounts)
    }

    pub fn new_for_account_contract(
        account_contract: FeatureContract,
        n_accounts: TransactionGeneratorAccountId,
    ) -> Self {
        if let FeatureContract::AccountWithoutValidations(CairoVersion::Cairo0) = account_contract {
            panic!("Cairo0 support untested, remove this check once support is verified.");
        }

        // *Generates* address for the `instance_id`'th account being tested.
        // TODO: rename `get_instance_address` to `generate_instance_address``
        let account_contracts = (0..n_accounts)
            .map(|account_id| (account_id, account_contract.get_instance_address(account_id)))
            .collect();

        Self { internal_nonce_manager: NonceManager::default(), account_contracts }
    }

    pub fn account(
        &mut self,
        account_id: TransactionGeneratorAccountId,
    ) -> AccountTransactionGenerator<'_> {
        AccountTransactionGenerator { account_id, generator: self }
    }
}

pub struct AccountTransactionGenerator<'a> {
    account_id: TransactionGeneratorAccountId,
    generator: &'a mut MultiAccountTransactionGenerator,
}

impl<'a> AccountTransactionGenerator<'a> {
    pub fn generate(&mut self, invoke_tx_args_overrides: InvokeTxArgs) -> ExternalTransaction {
        // Currently hardcoding the test contract, address, and calldata for calling a trivial
        // function in this test contract; when customization is needed, parameterize them.
        let default_test_contract = FeatureContract::TestContract(CairoVersion::Cairo1);
        let test_contract_address = default_test_contract.get_instance_address(0);

        let trivial_function_calldata_for_default_test_contract =
            create_trivial_calldata(test_contract_address);

        let sender_address = self.generator.account_contracts[&self.account_id];
        let nonce = self.generator.internal_nonce_manager.next(sender_address);

        external_invoke_tx(invoke_tx_args!(
            signature: TransactionSignature(vec![nonce.0]),
            sender_address,
            resource_bounds: executable_resource_bounds_mapping(),
            nonce,
            calldata: trivial_function_calldata_for_default_test_contract,
            ..invoke_tx_args_overrides
        ))
    }
}

// TODO(Ayelet, 28/5/2025): Try unifying the macros.
// TODO(Ayelet, 28/5/2025): Consider moving the macros StarkNet API.
#[macro_export]
macro_rules! invoke_tx_args {
    ($($field:ident $(: $value:expr)?),* $(,)?) => {
        $crate::starknet_api_test_utils::InvokeTxArgs {
            $($field $(: $value)?,)*
            ..Default::default()
        }
    };
    ($($field:ident $(: $value:expr)?),* , ..$defaults:expr) => {
        $crate::starknet_api_test_utils::InvokeTxArgs {
            $($field $(: $value)?,)*
            ..$defaults
        }
    };
}

#[macro_export]
macro_rules! deploy_account_tx_args {
    ($($field:ident $(: $value:expr)?),* $(,)?) => {
        $crate::starknet_api_test_utils::DeployAccountTxArgs {
            $($field $(: $value)?,)*
            ..Default::default()
        }
    };
    ($($field:ident $(: $value:expr)?),* , ..$defaults:expr) => {
        $crate::starknet_api_test_utils::DeployAccountTxArgs {
            $($field $(: $value)?,)*
            ..$defaults
        }
    };
}

#[macro_export]
macro_rules! declare_tx_args {
    ($($field:ident $(: $value:expr)?),* $(,)?) => {
        $crate::starknet_api_test_utils::DeclareTxArgs {
            $($field $(: $value)?,)*
            ..Default::default()
        }
    };
    ($($field:ident $(: $value:expr)?),* , ..$defaults:expr) => {
        $crate::starknet_api_test_utils::DeclareTxArgs {
            $($field $(: $value)?,)*
            ..$defaults
        }
    };
}

#[derive(Clone)]
pub struct InvokeTxArgs {
    pub signature: TransactionSignature,
    pub sender_address: ContractAddress,
    pub calldata: Calldata,
    pub version: TransactionVersion,
    pub resource_bounds: ResourceBoundsMapping,
    pub tip: Tip,
    pub nonce_data_availability_mode: DataAvailabilityMode,
    pub fee_data_availability_mode: DataAvailabilityMode,
    pub paymaster_data: PaymasterData,
    pub account_deployment_data: AccountDeploymentData,
    pub nonce: Nonce,
}

impl Default for InvokeTxArgs {
    fn default() -> Self {
        InvokeTxArgs {
            signature: TransactionSignature::default(),
            sender_address: ContractAddress::default(),
            calldata: calldata![],
            version: TransactionVersion::THREE,
            resource_bounds: zero_resource_bounds_mapping(),
            tip: Tip::default(),
            nonce_data_availability_mode: DataAvailabilityMode::L1,
            fee_data_availability_mode: DataAvailabilityMode::L1,
            paymaster_data: PaymasterData::default(),
            account_deployment_data: AccountDeploymentData::default(),
            nonce: Nonce::default(),
        }
    }
}

#[derive(Clone)]
pub struct DeployAccountTxArgs {
    pub signature: TransactionSignature,
    pub deployer_address: ContractAddress,
    pub version: TransactionVersion,
    pub resource_bounds: ResourceBoundsMapping,
    pub tip: Tip,
    pub nonce_data_availability_mode: DataAvailabilityMode,
    pub fee_data_availability_mode: DataAvailabilityMode,
    pub paymaster_data: PaymasterData,
    pub nonce: Nonce,
    pub class_hash: ClassHash,
    pub contract_address_salt: ContractAddressSalt,
    pub constructor_calldata: Calldata,
}

impl Default for DeployAccountTxArgs {
    fn default() -> Self {
        DeployAccountTxArgs {
            signature: TransactionSignature::default(),
            deployer_address: ContractAddress::default(),
            version: TransactionVersion::THREE,
            resource_bounds: zero_resource_bounds_mapping(),
            tip: Tip::default(),
            nonce_data_availability_mode: DataAvailabilityMode::L1,
            fee_data_availability_mode: DataAvailabilityMode::L1,
            paymaster_data: PaymasterData::default(),
            nonce: Nonce::default(),
            class_hash: ClassHash::default(),
            contract_address_salt: ContractAddressSalt::default(),
            constructor_calldata: Calldata::default(),
        }
    }
}

#[derive(Clone)]
pub struct DeclareTxArgs {
    pub signature: TransactionSignature,
    pub sender_address: ContractAddress,
    pub version: TransactionVersion,
    pub resource_bounds: ResourceBoundsMapping,
    pub tip: Tip,
    pub nonce_data_availability_mode: DataAvailabilityMode,
    pub fee_data_availability_mode: DataAvailabilityMode,
    pub paymaster_data: PaymasterData,
    pub account_deployment_data: AccountDeploymentData,
    pub nonce: Nonce,
    pub class_hash: CompiledClassHash,
    pub contract_class: ContractClass,
}

impl Default for DeclareTxArgs {
    fn default() -> Self {
        Self {
            signature: TransactionSignature::default(),
            sender_address: ContractAddress::default(),
            version: TransactionVersion::THREE,
            resource_bounds: zero_resource_bounds_mapping(),
            tip: Tip::default(),
            nonce_data_availability_mode: DataAvailabilityMode::L1,
            fee_data_availability_mode: DataAvailabilityMode::L1,
            paymaster_data: PaymasterData::default(),
            account_deployment_data: AccountDeploymentData::default(),
            nonce: Nonce::default(),
            class_hash: CompiledClassHash::default(),
            contract_class: ContractClass::default(),
        }
    }
}

pub fn external_invoke_tx(invoke_args: InvokeTxArgs) -> ExternalTransaction {
    match invoke_args.version {
        TransactionVersion::THREE => {
            starknet_api::external_transaction::ExternalTransaction::Invoke(
                starknet_api::external_transaction::ExternalInvokeTransaction::V3(
                    ExternalInvokeTransactionV3 {
                        resource_bounds: invoke_args.resource_bounds,
                        tip: invoke_args.tip,
                        calldata: invoke_args.calldata,
                        sender_address: invoke_args.sender_address,
                        nonce: invoke_args.nonce,
                        signature: invoke_args.signature,
                        nonce_data_availability_mode: invoke_args.nonce_data_availability_mode,
                        fee_data_availability_mode: invoke_args.fee_data_availability_mode,
                        paymaster_data: invoke_args.paymaster_data,
                        account_deployment_data: invoke_args.account_deployment_data,
                    },
                ),
            )
        }
        _ => panic!("Unsupported transaction version: {:?}.", invoke_args.version),
    }
}

pub fn external_deploy_account_tx(deploy_tx_args: DeployAccountTxArgs) -> ExternalTransaction {
    match deploy_tx_args.version {
        TransactionVersion::THREE => {
            starknet_api::external_transaction::ExternalTransaction::DeployAccount(
                starknet_api::external_transaction::ExternalDeployAccountTransaction::V3(
                    ExternalDeployAccountTransactionV3 {
                        resource_bounds: deploy_tx_args.resource_bounds,
                        tip: deploy_tx_args.tip,
                        contract_address_salt: deploy_tx_args.contract_address_salt,
                        class_hash: deploy_tx_args.class_hash,
                        constructor_calldata: deploy_tx_args.constructor_calldata,
                        nonce: deploy_tx_args.nonce,
                        signature: deploy_tx_args.signature,
                        nonce_data_availability_mode: deploy_tx_args.nonce_data_availability_mode,
                        fee_data_availability_mode: deploy_tx_args.fee_data_availability_mode,
                        paymaster_data: deploy_tx_args.paymaster_data,
                    },
                ),
            )
        }
        _ => panic!("Unsupported transaction version: {:?}.", deploy_tx_args.version),
    }
}

pub fn external_declare_tx(declare_tx_args: DeclareTxArgs) -> ExternalTransaction {
    match declare_tx_args.version {
        TransactionVersion::THREE => {
            starknet_api::external_transaction::ExternalTransaction::Declare(
                starknet_api::external_transaction::ExternalDeclareTransaction::V3(
                    ExternalDeclareTransactionV3 {
                        contract_class: declare_tx_args.contract_class,
                        signature: declare_tx_args.signature,
                        sender_address: declare_tx_args.sender_address,
                        resource_bounds: declare_tx_args.resource_bounds,
                        tip: declare_tx_args.tip,
                        nonce_data_availability_mode: declare_tx_args.nonce_data_availability_mode,
                        fee_data_availability_mode: declare_tx_args.fee_data_availability_mode,
                        paymaster_data: declare_tx_args.paymaster_data,
                        account_deployment_data: declare_tx_args.account_deployment_data,
                        nonce: declare_tx_args.nonce,
                        compiled_class_hash: declare_tx_args.class_hash,
                    },
                ),
            )
        }
        _ => panic!("Unsupported transaction version: {:?}.", declare_tx_args.version),
    }
}

pub fn external_tx_to_json(tx: &ExternalTransaction) -> String {
    let mut tx_json = serde_json::to_value(tx)
        .unwrap_or_else(|tx| panic!("Failed to serialize transaction: {tx:?}"));

    // Add type and version manually
    let type_string = match tx {
        ExternalTransaction::Declare(_) => "DECLARE",
        ExternalTransaction::DeployAccount(_) => "DEPLOY_ACCOUNT",
        ExternalTransaction::Invoke(_) => "INVOKE",
    };

    tx_json
        .as_object_mut()
        .unwrap()
        .extend([("type".to_string(), type_string.into()), ("version".to_string(), "0x3".into())]);

    // Serialize back to pretty JSON string
    to_string_pretty(&tx_json).expect("Failed to serialize transaction")
}
