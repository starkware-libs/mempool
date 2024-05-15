use blockifier::execution::contract_class::ClassInfo;
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::transactions::{
    DeclareTransaction as BlockifierDeclareTransaction,
    DeployAccountTransaction as BlockifierDeployAccountTransaction,
    InvokeTransaction as BlockifierInvokeTransaction,
};
use serde_json::{to_string_pretty, Value};
use starknet_api::calldata;
use starknet_api::core::{calculate_contract_address, ChainId, ClassHash, ContractAddress, Nonce};
use starknet_api::data_availability::DataAvailabilityMode;
use starknet_api::external_transaction::{
    ExternalDeclareTransaction, ExternalDeployAccountTransaction, ExternalInvokeTransaction,
    ExternalInvokeTransactionV3, ExternalTransaction,
};
use starknet_api::transaction::{
    AccountDeploymentData, Calldata, DeclareTransaction, DeclareTransactionV3,
    DeployAccountTransaction, DeployAccountTransactionV3, InvokeTransaction, InvokeTransactionV3,
    PaymasterData, Resource, ResourceBounds, ResourceBoundsMapping, Tip, TransactionHasher,
    TransactionSignature, TransactionVersion,
};

use crate::errors::StatefulTransactionValidatorResult;

macro_rules! implement_ref_getters {
    ($(($member_name:ident, $member_type:ty));* $(;)?) => {
        $(fn $member_name(&self) -> &$member_type {
            match self {
                starknet_api::external_transaction::ExternalTransaction::Declare(
                    starknet_api::external_transaction::ExternalDeclareTransaction::V3(tx)
                ) => &tx.$member_name,
                starknet_api::external_transaction::ExternalTransaction::DeployAccount(
                    starknet_api::external_transaction::ExternalDeployAccountTransaction::V3(tx)
                ) => &tx.$member_name,
                starknet_api::external_transaction::ExternalTransaction::Invoke(
                    starknet_api::external_transaction::ExternalInvokeTransaction::V3(tx)
                ) => &tx.$member_name,
            }
        })*
    };
}

impl ExternalTransactionExt for ExternalTransaction {
    implement_ref_getters!(
        (resource_bounds, ResourceBoundsMapping);
        (signature, TransactionSignature)
    );
}

// TODO(Arni, 1/5/2025): Remove this trait once it is implemented in StarkNet API.
pub trait ExternalTransactionExt {
    fn resource_bounds(&self) -> &ResourceBoundsMapping;
    fn signature(&self) -> &TransactionSignature;
}

pub fn external_tx_to_account_tx(
    external_tx: &ExternalTransaction,
    // FIXME(yael 15/4/24): calculate class_info inside the function once compilation code is ready
    optional_class_info: Option<ClassInfo>,
    chain_id: &ChainId,
) -> StatefulTransactionValidatorResult<AccountTransaction> {
    match external_tx {
        ExternalTransaction::Declare(ExternalDeclareTransaction::V3(tx)) => {
            let declare_tx = DeclareTransaction::V3(DeclareTransactionV3 {
                class_hash: ClassHash::default(), /* FIXME(yael 15/4/24): call the starknet-api
                                                   * function once ready */
                resource_bounds: tx.resource_bounds.clone(),
                tip: tx.tip,
                signature: tx.signature.clone(),
                nonce: tx.nonce,
                compiled_class_hash: tx.compiled_class_hash,
                sender_address: tx.sender_address,
                nonce_data_availability_mode: tx.nonce_data_availability_mode,
                fee_data_availability_mode: tx.fee_data_availability_mode,
                paymaster_data: tx.paymaster_data.clone(),
                account_deployment_data: tx.account_deployment_data.clone(),
            });
            let tx_hash = declare_tx.calculate_transaction_hash(chain_id, &declare_tx.version())?;
            let class_info =
                optional_class_info.expect("declare transaction should contain class info");
            let declare_tx = BlockifierDeclareTransaction::new(declare_tx, tx_hash, class_info)?;
            Ok(AccountTransaction::Declare(declare_tx))
        }
        ExternalTransaction::DeployAccount(ExternalDeployAccountTransaction::V3(tx)) => {
            let deploy_account_tx = DeployAccountTransaction::V3(DeployAccountTransactionV3 {
                resource_bounds: tx.resource_bounds.clone(),
                tip: tx.tip,
                signature: tx.signature.clone(),
                nonce: tx.nonce,
                class_hash: tx.class_hash,
                contract_address_salt: tx.contract_address_salt,
                constructor_calldata: tx.constructor_calldata.clone(),
                nonce_data_availability_mode: tx.nonce_data_availability_mode,
                fee_data_availability_mode: tx.fee_data_availability_mode,
                paymaster_data: tx.paymaster_data.clone(),
            });
            let contract_address = calculate_contract_address(
                deploy_account_tx.contract_address_salt(),
                deploy_account_tx.class_hash(),
                &deploy_account_tx.constructor_calldata(),
                ContractAddress::default(),
            )?;
            let tx_hash = deploy_account_tx
                .calculate_transaction_hash(chain_id, &deploy_account_tx.version())?;
            let deploy_account_tx = BlockifierDeployAccountTransaction::new(
                deploy_account_tx,
                tx_hash,
                contract_address,
            );
            Ok(AccountTransaction::DeployAccount(deploy_account_tx))
        }
        ExternalTransaction::Invoke(ExternalInvokeTransaction::V3(tx)) => {
            let invoke_tx = InvokeTransaction::V3(InvokeTransactionV3 {
                resource_bounds: tx.resource_bounds.clone(),
                tip: tx.tip,
                signature: tx.signature.clone(),
                nonce: tx.nonce,
                sender_address: tx.sender_address,
                calldata: tx.calldata.clone(),
                nonce_data_availability_mode: tx.nonce_data_availability_mode,
                fee_data_availability_mode: tx.fee_data_availability_mode,
                paymaster_data: tx.paymaster_data.clone(),
                account_deployment_data: tx.account_deployment_data.clone(),
            });
            let tx_hash = invoke_tx.calculate_transaction_hash(chain_id, &invoke_tx.version())?;
            let invoke_tx = BlockifierInvokeTransaction::new(invoke_tx, tx_hash);
            Ok(AccountTransaction::Invoke(invoke_tx))
        }
    }
}

#[derive(Clone)]
pub struct InvokeTxArgs {
    pub signature: TransactionSignature,
    pub contract_address: ContractAddress,
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
            contract_address: ContractAddress::default(),
            calldata: calldata![],
            version: TransactionVersion::THREE,
            resource_bounds: ResourceBoundsMapping::try_from(vec![
                (Resource::L1Gas, ResourceBounds { max_amount: 0, max_price_per_unit: 1 }),
                (Resource::L2Gas, ResourceBounds { max_amount: 0, max_price_per_unit: 0 }),
            ])
            .unwrap(),
            tip: Tip::default(),
            nonce_data_availability_mode: DataAvailabilityMode::L1,
            fee_data_availability_mode: DataAvailabilityMode::L1,
            paymaster_data: PaymasterData::default(),
            account_deployment_data: AccountDeploymentData::default(),
            nonce: Nonce::default(),
        }
    }
}

// TODO(Ayelet, 15/5/2025): Move this to StarkNet API.
#[macro_export]
macro_rules! invoke_tx_args {
    ($($field:ident $(: $value:expr)?),* $(,)?) => {
        $crate::utils::InvokeTxArgs {
            $($field $(: $value)?,)*
            ..Default::default()
        }
    };
    ($($field:ident $(: $value:expr)?),* , ..$defaults:expr) => {
        $crate::utils::InvokeTxArgs {
            $($field $(: $value)?,)*
            ..$defaults
        }
    };
}

pub fn external_invoke_tx(invoke_args: InvokeTxArgs) -> ExternalInvokeTransaction {
    match invoke_args.version {
        TransactionVersion::THREE => {
            starknet_api::external_transaction::ExternalInvokeTransaction::V3(
                ExternalInvokeTransactionV3 {
                    resource_bounds: invoke_args.resource_bounds,
                    tip: invoke_args.tip,
                    calldata: invoke_args.calldata,
                    sender_address: invoke_args.contract_address,
                    nonce: invoke_args.nonce,
                    signature: invoke_args.signature,
                    nonce_data_availability_mode: invoke_args.nonce_data_availability_mode,
                    fee_data_availability_mode: invoke_args.fee_data_availability_mode,
                    paymaster_data: invoke_args.paymaster_data,
                    account_deployment_data: invoke_args.account_deployment_data,
                },
            )
        }
        _ => panic!("Unsupported transaction version: {:?}.", invoke_args.version),
    }
}

pub fn external_invoke_tx_to_json(tx: ExternalTransaction) -> String {
    // Serialize to JSON
    let mut tx_json = serde_json::to_value(&tx).expect("Failed to serialize transaction");

    // Add type and version manually
    if let Value::Object(ref mut map) = tx_json {
        map.insert("type".to_string(), Value::String("INVOKE_FUNCTION".to_string()));
    }
    if let Value::Object(ref mut map) = tx_json {
        map.insert("version".to_string(), Value::String("0x3".to_string()));
    }

    // Serialize back to pretty JSON string
    to_string_pretty(&tx_json).expect("Failed to serialize transaction")
}
