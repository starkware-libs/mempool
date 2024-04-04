use strum::IntoEnumIterator;

use starknet_api::external_transaction::{
    ExternalDeclareTransaction, ExternalDeployAccountTransaction, ExternalInvokeTransaction,
    ExternalTransaction,
};
use starknet_api::transaction::Resource;

use crate::errors::{TransactionValidatorError, TransactionValidatorResult};

#[cfg(test)]
#[path = "transaction_validator_test.rs"]
mod transaction_validator_test;

#[derive(Default)]
pub struct TransactionValidatorConfig {
    // if true, validates that the reousrce bounds are not zero.
    pub validate_non_zero_l1_gas_fee: bool,
    pub validate_non_zero_l2_gas_fee: bool,
}

pub struct TransactionValidator {
    pub config: TransactionValidatorConfig,
}

impl TransactionValidator {
    pub fn validate(&self, tx: ExternalTransaction) -> TransactionValidatorResult<()> {
        // TODO(Arni, 1/5/2024): Add a mechanism that validate the sender address is not blocked.
        // TODO(Arni, 1/5/2024): Validate transaction version.
        // TODO(Arni, 4/4/2024): Validate tx signature and calldata are not too long.

        self.validate_fee(&tx)?;

        Ok(())
    }

    fn validate_fee(&self, tx: &ExternalTransaction) -> TransactionValidatorResult<()> {
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

        for resource in Resource::iter() {
            if (resource == Resource::L1Gas && !self.config.validate_non_zero_l1_gas_fee)
                || (resource == Resource::L2Gas && !self.config.validate_non_zero_l2_gas_fee)
            {
                continue;
            }
            let resource_bounds = resource_bounds_mapping.0.get(&resource);
            match resource_bounds {
                None => {
                    return Err(TransactionValidatorError::MissingResource { resource });
                }
                Some(resource_bounds) => {
                    if resource_bounds.max_amount == 0 || resource_bounds.max_price_per_unit == 0 {
                        return Err(TransactionValidatorError::ZeroFee {
                            resource,
                            resource_bounds: *resource_bounds,
                        });
                    }
                }
            }
        }

        Ok(())
    }
}
