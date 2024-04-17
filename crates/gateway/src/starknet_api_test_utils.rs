use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::data_availability::DataAvailabilityMode;
use starknet_api::external_transaction::{
    ExternalDeclareTransaction, ExternalDeclareTransactionV3, ExternalDeployAccountTransaction,
    ExternalDeployAccountTransactionV3, ExternalInvokeTransaction, ExternalInvokeTransactionV3,
    ExternalTransaction,
};
use starknet_api::transaction::{Calldata, ResourceBounds, ResourceBoundsMapping};

// Utils.
pub fn external_declare_tx_for_testing(
    resource_bounds: ResourceBoundsMapping,
) -> ExternalTransaction {
    ExternalTransaction::Declare(ExternalDeclareTransaction::V3(
        ExternalDeclareTransactionV3 {
            resource_bounds,
            contract_class: Default::default(),
            tip: Default::default(),
            signature: Default::default(),
            nonce: Default::default(),
            compiled_class_hash: Default::default(),
            sender_address: Default::default(),
            nonce_data_availability_mode: DataAvailabilityMode::L1,
            fee_data_availability_mode: DataAvailabilityMode::L1,
            paymaster_data: Default::default(),
            account_deployment_data: Default::default(),
        },
    ))
}

pub fn external_deploy_account_tx_for_testing(
    resource_bounds: ResourceBoundsMapping,
) -> ExternalTransaction {
    ExternalTransaction::DeployAccount(ExternalDeployAccountTransaction::V3(
        ExternalDeployAccountTransactionV3 {
            resource_bounds,
            tip: Default::default(),
            contract_address_salt: Default::default(),
            class_hash: Default::default(),
            constructor_calldata: Default::default(),
            nonce: Default::default(),
            signature: Default::default(),
            nonce_data_availability_mode: DataAvailabilityMode::L1,
            fee_data_availability_mode: DataAvailabilityMode::L1,
            paymaster_data: Default::default(),
        },
    ))
}

pub fn external_invoke_tx_for_testing(
    resource_bounds: ResourceBoundsMapping,
) -> ExternalTransaction {
    executable_external_invoke_tx_for_testing(
        resource_bounds,
        Nonce::default(),
        ContractAddress::default(),
        Calldata::default(),
    )
}

pub fn executable_external_invoke_tx_for_testing(
    resource_bounds: ResourceBoundsMapping,
    nonce: Nonce,
    sender_address: ContractAddress,
    calldata: Calldata,
) -> ExternalTransaction {
    ExternalTransaction::Invoke(ExternalInvokeTransaction::V3(ExternalInvokeTransactionV3 {
        resource_bounds,
        tip: Default::default(),
        signature: Default::default(),
        nonce,
        sender_address,
        calldata,
        nonce_data_availability_mode: DataAvailabilityMode::L1,
        fee_data_availability_mode: DataAvailabilityMode::L1,
        paymaster_data: Default::default(),
        account_deployment_data: Default::default(),
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

pub fn non_zero_l1_resource_bounds_mapping() -> ResourceBoundsMapping {
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

pub fn non_zero_l2_resource_bounds_mapping() -> ResourceBoundsMapping {
    ResourceBoundsMapping::try_from(vec![
        (
            starknet_api::transaction::Resource::L1Gas,
            ResourceBounds::default(),
        ),
        (
            starknet_api::transaction::Resource::L2Gas,
            ResourceBounds {
                max_amount: 1,
                max_price_per_unit: 1,
            },
        ),
    ])
    .expect("Resource bounds mapping has unexpected structure.")
}

pub const VALID_L1_GAS_MAX_AMOUNT: u64 = 1662;
pub const VALID_L1_GAS_MAX_PRICE_PER_UNIT: u128 = 100000000000;
pub fn executable_resource_bounds_mapping() -> ResourceBoundsMapping {
    ResourceBoundsMapping::try_from(vec![
        (
            starknet_api::transaction::Resource::L1Gas,
            ResourceBounds {
                max_amount: VALID_L1_GAS_MAX_AMOUNT,
                max_price_per_unit: VALID_L1_GAS_MAX_PRICE_PER_UNIT,
            },
        ),
        (
            starknet_api::transaction::Resource::L2Gas,
            ResourceBounds::default(),
        ),
    ])
    .expect("Resource bounds mapping has unexpected structure.")
}
