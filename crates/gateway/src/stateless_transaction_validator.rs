use starknet_api::external_transaction::{
    ExternalDeclareTransaction, ExternalDeployAccountTransaction, ExternalInvokeTransaction,
    ExternalTransaction,
};
use starknet_api::transaction::{Resource, ResourceBoundsMapping};

use crate::errors::{TransactionValidatorError, TransactionValidatorResult};

#[cfg(test)]
#[path = "stateless_transaction_validator_test.rs"]
mod transaction_validator_test;

#[derive(Clone)]
pub struct StatelessTransactionValidatorConfig {
    // If true, validates that the resource bounds are not zero.
    pub validate_non_zero_l1_gas_fee: bool,
    pub validate_non_zero_l2_gas_fee: bool,

    pub max_calldata_length: usize,
}

impl Default for StatelessTransactionValidatorConfig {
    fn default() -> Self {
        Self {
            validate_non_zero_l1_gas_fee: false,
            validate_non_zero_l2_gas_fee: false,
            max_calldata_length: 1000,
        }
    }
}

pub struct StatelessTransactionValidator {
    pub config: StatelessTransactionValidatorConfig,
}

impl StatelessTransactionValidator {
    pub fn validate(&self, tx: &ExternalTransaction) -> TransactionValidatorResult<()> {
        // TODO(Arni, 1/5/2024): Add a mechanism that validate the sender address is not blocked.
        // TODO(Arni, 1/5/2024): Validate transaction version.

        self.validate_fee(tx)?;
        self.validate_tx_size(tx)?;

        Ok(())
    }

    fn validate_fee(&self, tx: &ExternalTransaction) -> TransactionValidatorResult<()> {
        let resource_bounds_mapping = match tx {
            ExternalTransaction::Declare(ExternalDeclareTransaction::V3(tx)) => &tx.resource_bounds,
            ExternalTransaction::DeployAccount(ExternalDeployAccountTransaction::V3(tx)) => {
                &tx.resource_bounds
            }
            ExternalTransaction::Invoke(ExternalInvokeTransaction::V3(tx)) => &tx.resource_bounds,
        };

        if self.config.validate_non_zero_l1_gas_fee {
            validate_resource_bounds(resource_bounds_mapping, Resource::L1Gas)?;
        }
        if self.config.validate_non_zero_l2_gas_fee {
            validate_resource_bounds(resource_bounds_mapping, Resource::L2Gas)?;
        }

        Ok(())
    }

    fn validate_tx_size(&self, tx: &ExternalTransaction) -> TransactionValidatorResult<()> {
        self.validate_tx_calldata_size(tx)?;

        // TODO(Arni, 4/4/2024): Validate tx signature is not too long.

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
            ExternalTransaction::DeployAccount(tx) => match tx {
                ExternalDeployAccountTransaction::V3(tx) => &tx.constructor_calldata,
            },
            ExternalTransaction::Invoke(tx) => match tx {
                ExternalInvokeTransaction::V3(tx) => &tx.calldata,
            },
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
}

// Utilities.

fn validate_resource_bounds(
    resource_bounds_mapping: &ResourceBoundsMapping,
    resource: Resource,
) -> TransactionValidatorResult<()> {
    if let Some(resource_bounds) = resource_bounds_mapping.0.get(&resource) {
        if resource_bounds.max_amount == 0 || resource_bounds.max_price_per_unit == 0 {
            return Err(TransactionValidatorError::ZeroFee {
                resource,
                resource_bounds: *resource_bounds,
            });
        }
    } else {
        return Err(TransactionValidatorError::MissingResource { resource });
    }

    Ok(())
}
