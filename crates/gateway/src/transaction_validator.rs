use starknet_api::external_transaction::{
    ExternalDeclareTransaction, ExternalDeployAccountTransaction, ExternalInvokeTransaction,
    ExternalTransaction,
};
use starknet_api::transaction::Resource;

use crate::errors::{TransactionValidatorError, TransactionValidatorResult};

#[cfg(test)]
#[path = "transaction_validator_test.rs"]
mod transaction_validator_test;

// TODO(Arni, 1/5/2024): Remove the this value if we use a type which supports version 3 and above
// only.
// It is an assumption of this repository that the minimal supported transaction version is 3.

pub struct TransactionValidatorConfig {
    pub fee_resource: Resource,

    pub max_calldata_length: usize,
    pub max_signature_length: usize,
}

impl Default for TransactionValidatorConfig {
    fn default() -> Self {
        Self {
            fee_resource: Resource::L1Gas,
            max_calldata_length: Default::default(),
            max_signature_length: Default::default(),
        }
    }
}

pub struct TransactionValidator {
    pub config: TransactionValidatorConfig,
}

impl TransactionValidator {
    pub fn validate(&self, tx: ExternalTransaction) -> TransactionValidatorResult<()> {
        // TODO(Arni, 1/5/2024): Add a mechanism that validate the sender address is not blocked.
        // TODO(Arni, 1/5/2024): Validate transaction version.

        self.validate_fee(&tx)?;
        self.validate_tx_size(&tx)?;

        Ok(())
    }

    fn validate_fee(&self, tx: &ExternalTransaction) -> TransactionValidatorResult<()> {
        let resource = self.config.fee_resource;
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
        let resource_bounds = resource_bounds_mapping.0[&resource];
        if resource_bounds.max_amount == 0 || resource_bounds.max_price_per_unit == 0 {
            return Err(TransactionValidatorError::ZeroFee {
                resource,
                resource_bounds,
            });
        }

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

    fn validate_tx_signature_size(
        &self,
        tx: &ExternalTransaction,
    ) -> TransactionValidatorResult<()> {
        let signature = match tx {
            ExternalTransaction::Declare(tx) => match tx {
                ExternalDeclareTransaction::V3(tx) => &tx.signature,
            },
            ExternalTransaction::DeployAccount(tx) => match tx {
                ExternalDeployAccountTransaction::V3(tx) => &tx.signature,
            },
            ExternalTransaction::Invoke(tx) => match tx {
                ExternalInvokeTransaction::V3(tx) => &tx.signature,
            },
        };

        let signature_length = signature.0.len();
        if signature_length > self.config.max_signature_length {
            return Err(TransactionValidatorError::SignatureTooLong {
                signature_length,
                max_signature_length: self.config.max_signature_length,
            });
        }

        Ok(())
    }

    fn validate_tx_size(&self, tx: &ExternalTransaction) -> TransactionValidatorResult<()> {
        self.validate_tx_calldata_size(tx)?;
        self.validate_tx_signature_size(tx)?;

        Ok(())
    }
}
