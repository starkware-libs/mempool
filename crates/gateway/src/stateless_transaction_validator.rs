use starknet_api::external_transaction::{
    ExternalDeclareTransaction, ExternalDeployAccountTransaction, ExternalInvokeTransaction,
    ExternalTransaction,
};
use starknet_api::transaction::{Resource, ResourceBoundsMapping, TransactionSignature};

use crate::errors::{TransactionValidatorError, TransactionValidatorResult};

#[cfg(test)]
#[path = "stateless_transaction_validator_test.rs"]
mod transaction_validator_test;

#[derive(Default)]
pub struct StatelessTransactionValidatorConfig {
    // If true, validates that the resource bounds are not zero.
    pub validate_non_zero_l1_gas_fee: bool,
    pub validate_non_zero_l2_gas_fee: bool,

    pub max_calldata_length: usize,
    pub max_signature_length: usize,
}

pub struct StatelessTransactionValidator {
    pub config: StatelessTransactionValidatorConfig,
}

impl StatelessTransactionValidator {
    pub fn validate(&self, tx: &ExternalTransaction) -> TransactionValidatorResult<()> {
        // TODO(Arni, 1/5/2024): Add a mechanism that validate the sender address is not blocked.
        // TODO(Arni, 1/5/2024): Validate transaction version.

        self.validate_resource_bounds(tx)?;
        self.validate_tx_size(tx)?;

        Ok(())
    }

    fn validate_resource_bounds(&self, tx: &ExternalTransaction) -> TransactionValidatorResult<()> {
        let resource_bounds_mapping = tx.resource_bounds();

        if self.config.validate_non_zero_l1_gas_fee {
            validate_resource_is_non_zero(resource_bounds_mapping, Resource::L1Gas)?;
        }
        if self.config.validate_non_zero_l2_gas_fee {
            validate_resource_is_non_zero(resource_bounds_mapping, Resource::L2Gas)?;
        }

        Ok(())
    }

    fn validate_tx_size(&self, tx: &ExternalTransaction) -> TransactionValidatorResult<()> {
        self.validate_tx_calldata_size(tx)?;
        self.validate_tx_signature_size(tx)?;

        Ok(())
    }

    fn validate_tx_calldata_size(
        &self,
        tx: &ExternalTransaction,
    ) -> TransactionValidatorResult<()> {
        let calldata = match tx {
            ExternalTransaction::Declare(_) => {
                // Declare transaction has no calldata.
                return Ok(());
            }
            ExternalTransaction::DeployAccount(ExternalDeployAccountTransaction::V3(tx)) => {
                &tx.constructor_calldata
            }
            ExternalTransaction::Invoke(ExternalInvokeTransaction::V3(tx)) => &tx.calldata,
        };

        let calldata_length = calldata.0.len();
        if calldata_length > self.config.max_calldata_length {
            return Err(TransactionValidatorError::CalldataTooLong {
                calldata_length,
                max_calldata_length: self.config.max_calldata_length,
            });
        }

        Ok(())
    }

    fn validate_tx_signature_size(
        &self,
        tx: &ExternalTransaction,
    ) -> TransactionValidatorResult<()> {
        let signature = tx.signature();

        let signature_length = signature.0.len();
        if signature_length > self.config.max_signature_length {
            return Err(TransactionValidatorError::SignatureTooLong {
                signature_length,
                max_signature_length: self.config.max_signature_length,
            });
        }

        Ok(())
    }
}

// Utilities.

fn validate_resource_is_non_zero(
    resource_bounds_mapping: &ResourceBoundsMapping,
    resource: Resource,
) -> TransactionValidatorResult<()> {
    if let Some(resource_bounds) = resource_bounds_mapping.0.get(&resource) {
        if resource_bounds.max_amount == 0 || resource_bounds.max_price_per_unit == 0 {
            return Err(TransactionValidatorError::ZeroResourceBounds {
                resource,
                resource_bounds: *resource_bounds,
            });
        }
    } else {
        return Err(TransactionValidatorError::MissingResource { resource });
    }

    Ok(())
}

macro_rules! implement_ref_getters {
    ($(($member_name:ident, $member_type:ty));* $(;)?) => {
        $(fn $member_name(&self) -> &$member_type {
            match self {
                ExternalTransaction::Declare(ExternalDeclareTransaction::V3(tx)) => {
                    &tx.$member_name
                }
                ExternalTransaction::DeployAccount(ExternalDeployAccountTransaction::V3(tx)) => {
                    &tx.$member_name
                }
                ExternalTransaction::Invoke(ExternalInvokeTransaction::V3(tx)) => &tx.$member_name,
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

trait ExternalTransactionExt {
    fn resource_bounds(&self) -> &ResourceBoundsMapping;
    fn signature(&self) -> &TransactionSignature;
}
