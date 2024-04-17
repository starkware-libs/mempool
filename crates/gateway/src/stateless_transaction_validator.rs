use starknet_api::external_transaction::{
    ExternalDeclareTransaction, ExternalDeployAccountTransaction, ExternalInvokeTransaction,
    ExternalTransaction,
};
use starknet_api::transaction::Resource;

use crate::errors::{StatelessTransactionValidatorError, StatelessTransactionValidatorResult};

#[cfg(test)]
#[path = "stateless_transaction_validator_test.rs"]
mod transaction_validator_test;

#[derive(Default)]
pub struct StatelessTransactionValidatorConfig {
    // If true, validates that the reousrce bounds are not zero.
    pub validate_non_zero_l1_gas_fee: bool,
    pub validate_non_zero_l2_gas_fee: bool,
}

pub struct StatelessTransactionValidator {
    pub config: StatelessTransactionValidatorConfig,
}

impl StatelessTransactionValidator {
    pub fn validate(&self, tx: &ExternalTransaction) -> StatelessTransactionValidatorResult<()> {
        // TODO(Arni, 1/5/2024): Add a mechanism that validate the sender address is not blocked.
        // TODO(Arni, 1/5/2024): Validate transaction version.
        // TODO(Arni, 4/4/2024): Validate tx signature and calldata are not too long.

        self.validate_fee(tx)?;

        Ok(())
    }

    fn validate_fee(&self, tx: &ExternalTransaction) -> StatelessTransactionValidatorResult<()> {
        let resource_bounds_mapping = match tx {
            ExternalTransaction::Declare(tx) => match tx {
                ExternalDeclareTransaction::V3(tx) => &tx.resource_bounds,
            },
            ExternalTransaction::DeployAccount(tx) => match tx {
                ExternalDeployAccountTransaction::V3(tx) => &tx.resource_bounds,
            },
            ExternalTransaction::Invoke(tx) => match tx {
                ExternalInvokeTransaction::V3(tx) => &tx.resource_bounds,
            },
        };

        fn validate_reousrce_bounds(
            resource_bounds_mapping: &starknet_api::transaction::ResourceBoundsMapping,
            resource: Resource,
        ) -> StatelessTransactionValidatorResult<()> {
            if let Some(resource_bounds) = resource_bounds_mapping.0.get(&resource) {
                if resource_bounds.max_amount == 0 || resource_bounds.max_price_per_unit == 0 {
                    return Err(StatelessTransactionValidatorError::ZeroFee {
                        resource,
                        resource_bounds: *resource_bounds,
                    });
                }
            } else {
                return Err(StatelessTransactionValidatorError::MissingResource { resource });
            }

            Ok(())
        }

        if self.config.validate_non_zero_l1_gas_fee {
            validate_reousrce_bounds(resource_bounds_mapping, Resource::L1Gas)?;
        }
        if self.config.validate_non_zero_l2_gas_fee {
            validate_reousrce_bounds(resource_bounds_mapping, Resource::L2Gas)?;
        }

        Ok(())
    }
}
