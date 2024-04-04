use starknet_api::external_transaction::{
    DataAvailabilityMode, ExternalDeclareTransaction, ExternalDeclareTransactionV3,
    ExternalDeployAccountTransaction, ExternalDeployAccountTransactionV3,
    ExternalInvokeTransaction, ExternalInvokeTransactionV3, ExternalTransaction,
};
use starknet_api::transaction::{
    Calldata, ResourceBounds, ResourceBoundsMapping, TransactionSignature, TransactionVersion,
};

// Utils.
pub fn create_external_declare_tx_for_testing(
    resource_bounds: ResourceBoundsMapping,
    signature: TransactionSignature,
) -> ExternalTransaction {
    ExternalTransaction::Declare(ExternalDeclareTransaction::V3(
        ExternalDeclareTransactionV3 {
            resource_bounds,
            contract_class: Default::default(),
            tip: Default::default(),
            signature,
            nonce: Default::default(),
            compiled_class_hash: Default::default(),
            sender_address: Default::default(),
            nonce_data_availability_mode: DataAvailabilityMode::L1,
            fee_data_availability_mode: DataAvailabilityMode::L1,
            paymaster_data: Default::default(),
            account_deployment_data: Default::default(),
            version: TransactionVersion::THREE,
            r#type: starknet_api::external_transaction::DeclareType::Declare,
        },
    ))
}

pub fn create_external_deploy_account_tx_for_testing(
    resource_bounds: ResourceBoundsMapping,
    constructor_calldata: Calldata,
    signature: TransactionSignature,
) -> ExternalTransaction {
    ExternalTransaction::DeployAccount(ExternalDeployAccountTransaction::V3(
        ExternalDeployAccountTransactionV3 {
            resource_bounds,
            tip: Default::default(),
            contract_address_salt: Default::default(),
            class_hash: Default::default(),
            constructor_calldata,
            nonce: Default::default(),
            signature,
            nonce_data_availability_mode: DataAvailabilityMode::L1,
            fee_data_availability_mode: DataAvailabilityMode::L1,
            paymaster_data: Default::default(),
            version: TransactionVersion::THREE,
            r#type: starknet_api::external_transaction::DeployAccountType::DeployAccount,
        },
    ))
}

pub fn create_external_invoke_tx_for_testing(
    resource_bounds: ResourceBoundsMapping,
    calldata: Calldata,
    signature: TransactionSignature,
) -> ExternalTransaction {
    ExternalTransaction::Invoke(ExternalInvokeTransaction::V3(ExternalInvokeTransactionV3 {
        resource_bounds,
        tip: Default::default(),
        signature,
        nonce: Default::default(),
        sender_address: Default::default(),
        calldata,
        nonce_data_availability_mode: DataAvailabilityMode::L1,
        fee_data_availability_mode: DataAvailabilityMode::L1,
        paymaster_data: Default::default(),
        account_deployment_data: Default::default(),
        version: TransactionVersion::THREE,
        r#type: starknet_api::external_transaction::InvokeType::Invoke,
    }))
}

pub fn zero_resource_bounds_mapping() -> ResourceBoundsMapping {
    ResourceBoundsMapping::try_from(vec![
        (
            starknet_api::transaction::Resource::L1Gas,
            ResourceBounds::default(),
        ),
        (
            starknet_api::transaction::Resource::L2Gas,
            ResourceBounds::default(),
        ),
    ])
    .expect("Resource bounds mapping has unexpected structure.")
}

pub fn non_zero_resource_bounds_mapping() -> ResourceBoundsMapping {
    ResourceBoundsMapping::try_from(vec![
        (
            starknet_api::transaction::Resource::L1Gas,
            ResourceBounds {
                max_amount: 1,
                max_price_per_unit: 1,
            },
        ),
        (
            starknet_api::transaction::Resource::L2Gas,
            ResourceBounds::default(),
        ),
    ])
    .expect("Resource bounds mapping has unexpected structure.")
}
