use starknet_api::transaction::{Resource, Transaction, TransactionVersion};

use crate::errors::{TransactionValidatorError, TransactionValidatorResult};
use crate::starknet_api_utils::{TransactionParametersExt, TransactionVersionExt};

#[cfg(test)]
#[path = "transaction_validator_test.rs"]
mod transaction_validator_test;

// It is an assumption of this repository that the minimal supported transaction version is 3.
const HARD_CODED_MINIMAL_TX_VERSION: TransactionVersion = TransactionVersion::THREE;

pub struct TransactionValidatorConfig {
    pub block_declare_cairo1: bool,
    pub block_declare_cairo0: bool,

    pub min_allowed_tx_version: TransactionVersion,
    pub max_allowed_tx_version: TransactionVersion,

    pub fee_resource: Resource,

    pub max_calldata_length: usize,
    pub max_signature_length: usize,
}

impl Default for TransactionValidatorConfig {
    fn default() -> Self {
        Self {
            block_declare_cairo1: Default::default(),
            block_declare_cairo0: Default::default(),
            min_allowed_tx_version: Default::default(),
            max_allowed_tx_version: Default::default(),
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
    pub fn validate(&self, tx: Transaction) -> TransactionValidatorResult<()> {
        // Deploy transactions are deprecated.
        // L1Handler transactions are not supported in the gateway.
        if matches!(tx, Transaction::Deploy(_) | Transaction::L1Handler(_)) {
            return Err(TransactionValidatorError::TransactionTypeNotSupported);
        }

        // Check if the declaration of Cairo / Cairo-0 contracts are blocked.
        if let Transaction::Declare(tx) = &tx {
            if tx.version() < TransactionVersion::TWO && self.config.block_declare_cairo0 {
                return Err(TransactionValidatorError::BlockedTransactionVersion(
                    tx.version(),
                    "Declare of Cairo 0 is blocked.".into(),
                ));
            }
            if tx.version() >= TransactionVersion::TWO && self.config.block_declare_cairo1 {
                return Err(TransactionValidatorError::BlockedTransactionVersion(
                    tx.version(),
                    "Transaction type is temporarily blocked.".into(),
                ));
            }
        }

        // TODO(Arni, 1/4/2024): Add a mechanism that validate the sender address is not blocked.
        let version = tx.version();
        if version < self.config.min_allowed_tx_version {
            return Err(TransactionValidatorError::InvalidTransactionVersion(
                version,
                format!(
                    "Minimal supported version is {:?}.",
                    self.config.min_allowed_tx_version
                ),
            ));
        }
        if version > self.config.max_allowed_tx_version {
            return Err(TransactionValidatorError::InvalidTransactionVersion(
                version,
                format!(
                    "Maximal supported version is {:?}.",
                    self.config.max_allowed_tx_version
                ),
            ));
        }

        if version < HARD_CODED_MINIMAL_TX_VERSION {
            return Err(TransactionValidatorError::TransactionTypeNotSupported);
        }

        self.validate_fee(&tx)?;
        self.validate_tx_size(&tx)?;

        Ok(())
    }

    fn validate_fee(&self, tx: &Transaction) -> TransactionValidatorResult<()> {
        let resource = self.config.fee_resource;
        let resource_bounds = tx.ref_to_resource_bounds()?.0[&resource];
        if resource_bounds.max_amount == 0 || resource_bounds.max_price_per_unit == 0 {
            return Err(TransactionValidatorError::ZeroFee {
                resource,
                resource_bounds,
            });
        }

        Ok(())
    }

    fn validate_tx_size(&self, tx: &Transaction) -> TransactionValidatorResult<()> {
        if let Transaction::DeployAccount(_) | Transaction::Invoke(_) = tx {
            let calldata = tx.ref_to_calldata()?;
            if calldata.0.len() > self.config.max_calldata_length {
                return Err(TransactionValidatorError::CalldataTooLong {
                    calldata_length: calldata.0.len(),
                    max_calldata_length: self.config.max_calldata_length,
                });
            }
        }

        let signature = tx.ref_to_signature()?;
        if signature.0.len() > self.config.max_signature_length {
            return Err(TransactionValidatorError::SignatureTooLong {
                signature_length: signature.0.len(),
                max_signature_length: self.config.max_signature_length,
            });
        }

        Ok(())
    }
}
