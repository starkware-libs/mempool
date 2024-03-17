use starknet_api::{
    calldata,
    data_availability::DataAvailabilityMode,
    transaction::{
        Calldata, DeclareTransaction, DeclareTransactionV3, DeployAccountTransaction,
        DeployAccountTransactionV3, InvokeTransaction, InvokeTransactionV3, ResourceBounds,
        ResourceBoundsMapping, Transaction, TransactionVersion,
    },
};

// Traits.
trait CreateEmptyExt {
    fn create_empty() -> Self;
}

pub trait TransactionVersionExt {
    fn version(&self) -> TransactionVersion;
}

pub trait TransactionParametersExt {
    fn create_for_testing(resource_bounds: Option<ResourceBoundsMapping>) -> Self;
}

// Implementations.

impl CreateEmptyExt for ResourceBoundsMapping {
    fn create_empty() -> Self {
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
}

impl TransactionVersionExt for Transaction {
    fn version(&self) -> TransactionVersion {
        match self {
            Transaction::Declare(tx) => tx.version(),
            Transaction::DeployAccount(tx) => tx.version(),
            Transaction::Invoke(tx) => tx.version(),
            Transaction::Deploy(_) => TransactionVersion::ZERO,
            Transaction::L1Handler(_) => TransactionVersion::ZERO,
        }
    }
}

impl TransactionParametersExt for Transaction {
    fn create_for_testing(resource_bounds: Option<ResourceBoundsMapping>) -> Self {
        Self::Invoke(InvokeTransaction::create_for_testing(resource_bounds))
    }
}

impl TransactionParametersExt for DeclareTransaction {
    fn create_for_testing(resource_bounds: Option<ResourceBoundsMapping>) -> Self {
        DeclareTransaction::V3(DeclareTransactionV3 {
            resource_bounds: resource_bounds.unwrap_or(ResourceBoundsMapping::create_empty()),
            tip: Default::default(),
            signature: Default::default(),
            nonce: Default::default(),
            class_hash: Default::default(),
            compiled_class_hash: Default::default(),
            sender_address: Default::default(),
            nonce_data_availability_mode: DataAvailabilityMode::L1,
            fee_data_availability_mode: DataAvailabilityMode::L1,
            paymaster_data: Default::default(),
            account_deployment_data: Default::default(),
        })
    }
}

impl TransactionParametersExt for DeployAccountTransaction {
    fn create_for_testing(resource_bounds: Option<ResourceBoundsMapping>) -> Self {
        DeployAccountTransaction::V3(DeployAccountTransactionV3 {
            resource_bounds: resource_bounds.unwrap_or(ResourceBoundsMapping::create_empty()),
            tip: Default::default(),
            signature: Default::default(),
            nonce: Default::default(),
            class_hash: Default::default(),
            contract_address_salt: Default::default(),
            constructor_calldata: calldata![],
            nonce_data_availability_mode: DataAvailabilityMode::L1,
            fee_data_availability_mode: DataAvailabilityMode::L1,
            paymaster_data: Default::default(),
        })
    }
}

impl TransactionParametersExt for InvokeTransaction {
    fn create_for_testing(resource_bounds: Option<ResourceBoundsMapping>) -> Self {
        InvokeTransaction::V3(InvokeTransactionV3 {
            resource_bounds: resource_bounds.unwrap_or(ResourceBoundsMapping::create_empty()),
            tip: Default::default(),
            signature: Default::default(),
            nonce: Default::default(),
            sender_address: Default::default(),
            calldata: calldata![],
            nonce_data_availability_mode: DataAvailabilityMode::L1,
            fee_data_availability_mode: DataAvailabilityMode::L1,
            paymaster_data: Default::default(),
            account_deployment_data: Default::default(),
        })
    }
}
