use crate::errors::StatefulTransactionValidatorResult;
use blockifier::execution::contract_class::ClassInfo;
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::transactions::DeclareTransaction as BlockifierDeclareTransaction;
use blockifier::transaction::transactions::DeployAccountTransaction as BlockifierDeployAccountTransaction;
use blockifier::transaction::transactions::InvokeTransaction as BlockifierInvokeTransaction;
use starknet_api::core::{calculate_contract_address, ClassHash, ContractAddress};
use starknet_api::external_transaction::{
    ExternalDeclareTransaction, ExternalDeployAccountTransaction, ExternalInvokeTransaction,
    ExternalTransaction,
};
use starknet_api::transaction::DeclareTransaction;
use starknet_api::transaction::{
    DeclareTransactionV3, DeployAccountTransaction, DeployAccountTransactionV3, InvokeTransaction,
    InvokeTransactionV3, TransactionHash,
};

pub fn external_tx_to_account_tx(
    external_tx: &ExternalTransaction,
    optional_class_info: Option<ClassInfo>,
) -> StatefulTransactionValidatorResult<AccountTransaction> {
    let tx_hash = TransactionHash::default(); //FIXME(yael 15/4/24): make TransactionHasher public in starknet-api
    match external_tx {
        ExternalTransaction::Declare(ExternalDeclareTransaction::V3(tx)) => {
            let declare_tx = DeclareTransaction::V3(DeclareTransactionV3 {
                class_hash: ClassHash::default(), //FIXME(yael 15/4/24): call the starknet-api function once ready
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
            let invoke_tx = BlockifierInvokeTransaction::new(invoke_tx, tx_hash);
            Ok(AccountTransaction::Invoke(invoke_tx))
        }
    }
}
