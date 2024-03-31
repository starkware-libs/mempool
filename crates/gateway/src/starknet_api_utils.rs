use starknet_api::data_availability::DataAvailabilityMode;
use starknet_api::transaction::{
    Calldata, DeclareTransaction, DeclareTransactionV3, DeployAccountTransaction,
    DeployAccountTransactionV3, InvokeTransaction, InvokeTransactionV3, ResourceBounds,
    ResourceBoundsMapping, Transaction, TransactionSignature, TransactionVersion,
};

use crate::errors::{TransactionValidatorError, TransactionValidatorResult};

// Traits.
pub trait TransactionVersionExt {
    fn version(&self) -> TransactionVersion;
}

pub trait TransactionParametersExt {
    // Note, not all transaction types has a calldata field.
    fn ref_to_calldata(&self) -> TransactionValidatorResult<&Calldata>;
    fn ref_to_signature(&self) -> TransactionValidatorResult<&TransactionSignature>;
    fn create_for_testing(
        resource_bounds: ResourceBoundsMapping,
        signature: Option<TransactionSignature>,
        calldata: Option<Calldata>,
    ) -> Self;
}

// Macros.

macro_rules! implement_transaction_params_getters {
    ($(($ref_to_field:ident, $field_type:ty)),*) => {
        $(
            fn $ref_to_field(&self) -> TransactionValidatorResult<&$field_type> {
                match self {
                    Transaction::Declare(declare_tx) => declare_tx.$ref_to_field(),
                    Transaction::DeployAccount(deploy_account_tx) => deploy_account_tx.$ref_to_field(),
                    Transaction::Invoke(invoke_tx) => invoke_tx.$ref_to_field(),
                    _ => Err(TransactionValidatorError::TransactionTypeNotSupported),
                }
            }
        )*
    };
}

macro_rules! implement_tx_params_ref_getters {
    ($(($ref_to_field:ident, $field:ident, $field_type:ty)),*) => {
        $(
            fn $ref_to_field(&self) -> TransactionValidatorResult<&$field_type> {
                match self {
                    Self::V3(tx) => Ok(&tx.$field),
                    _ => unimplemented!("The gateway does not support this transaction version."),
                }
            }
        )*
    };
}

// Implementations.
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
    implement_transaction_params_getters!(
        (ref_to_calldata, Calldata),
        (ref_to_signature, TransactionSignature)
    );

    fn create_for_testing(
        resource_bounds: ResourceBoundsMapping,
        signature: Option<TransactionSignature>,
        calldata: Option<Calldata>,
    ) -> Self {
        Self::Invoke(InvokeTransaction::create_for_testing(
            resource_bounds,
            signature,
            calldata,
        ))
    }
}

impl TransactionParametersExt for DeclareTransaction {
    fn ref_to_calldata(&self) -> TransactionValidatorResult<&Calldata> {
        unimplemented!("Declare transactions do not have calldata.")
    }

    implement_tx_params_ref_getters!((ref_to_signature, signature, TransactionSignature));

    fn create_for_testing(
        resource_bounds: ResourceBoundsMapping,
        signature: Option<TransactionSignature>,
        calldata: Option<Calldata>,
    ) -> Self {
        assert!(calldata.is_none());
        DeclareTransaction::V3(DeclareTransactionV3 {
            resource_bounds,
            tip: Default::default(),
            signature: signature.unwrap_or_default(),
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
    implement_tx_params_ref_getters!(
        (ref_to_calldata, constructor_calldata, Calldata),
        (ref_to_signature, signature, TransactionSignature)
    );

    fn create_for_testing(
        resource_bounds: ResourceBoundsMapping,
        signature: Option<TransactionSignature>,
        calldata: Option<Calldata>,
    ) -> Self {
        DeployAccountTransaction::V3(DeployAccountTransactionV3 {
            resource_bounds,
            tip: Default::default(),
            signature: signature.unwrap_or_default(),
            nonce: Default::default(),
            class_hash: Default::default(),
            contract_address_salt: Default::default(),
            constructor_calldata: calldata.unwrap_or_default(),
            nonce_data_availability_mode: DataAvailabilityMode::L1,
            fee_data_availability_mode: DataAvailabilityMode::L1,
            paymaster_data: Default::default(),
        })
    }
}

impl TransactionParametersExt for InvokeTransaction {
    implement_tx_params_ref_getters!(
        (ref_to_calldata, calldata, Calldata),
        (ref_to_signature, signature, TransactionSignature)
    );

    fn create_for_testing(
        resource_bounds: ResourceBoundsMapping,
        signature: Option<TransactionSignature>,
        calldata: Option<Calldata>,
    ) -> Self {
        InvokeTransaction::V3(InvokeTransactionV3 {
            resource_bounds,
            tip: Default::default(),
            signature: signature.unwrap_or_default(),
            nonce: Default::default(),
            sender_address: Default::default(),
            calldata: calldata.unwrap_or_default(),
            nonce_data_availability_mode: DataAvailabilityMode::L1,
            fee_data_availability_mode: DataAvailabilityMode::L1,
            paymaster_data: Default::default(),
            account_deployment_data: Default::default(),
        })
    }
}

// Utils.

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
