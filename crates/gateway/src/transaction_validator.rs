use crate::errors::GatewayError;
use crate::errors::TransactionValidatorError;
use starknet_api::transaction::Transaction;
use starknet_api::transaction::TransactionVersion;

#[cfg(test)]
#[path = "transaction_validator_test.rs"]
mod transaction_validator_test;

#[derive(Default)]
pub struct TransactionValidatorConfig {
    pub block_declare_cairo1: bool,
    pub block_declare_cairo0: bool,

    pub min_allowed_tx_version: usize,
    pub max_allowed_tx_version: usize,
    pub current_tx_version: usize, // Should this constant be a part of the config?
}

pub struct TransactionValidator {
    pub config: TransactionValidatorConfig,
}

impl TransactionValidator {
    pub fn validate(&self, tx: Transaction) -> Result<(), GatewayError> {
        // Deploy and L1Handler transactions are not supported in the gateway.
        match tx {
            Transaction::Deploy(_) | Transaction::L1Handler(_) => {
                // Deploy transactions was deprecated.
                // L1Handler transactions are not supported in the gateway.
                return Err(TransactionValidatorError::InvalidTransactionType.into());
            }
            _ => {}
        }

        // Check if the declaration of Cairo / Cairo-0 contracts are blocked.
        if let Transaction::Declare(tx) = &tx {
            match tx.version() {
                TransactionVersion::ZERO | TransactionVersion::ONE => {
                    if self.config.block_declare_cairo0 {
                        return Err(TransactionValidatorError::BlockedTransactionVersion(
                            tx.version(),
                            "Declare of Cairo 0 is blocked.".into(),
                        )
                        .into());
                    }
                }
                TransactionVersion::TWO | TransactionVersion::THREE => {
                    if self.config.block_declare_cairo1 {
                        return Err(TransactionValidatorError::BlockedTransactionVersion(
                            tx.version(),
                            "Transaction type is temporarily blocked.".into(),
                        )
                        .into());
                    }
                }
                _ => {} // Invalid version will be handled later.
            }
        }

        // TODO(Arni, 1/4/2024): Add a mechanism that validate the sender address is not blocked.
        let version = get_tx_version(&tx);
        let version_as_usize: usize = version.0.try_into().expect("Invalid version.");
        if version_as_usize < self.config.min_allowed_tx_version {
            return Err(TransactionValidatorError::InvalidTransactionVersion(
                version,
                format!(
                    "Minimal supported version is {}.",
                    self.config.min_allowed_tx_version
                ),
            )
            .into());
        }
        if version_as_usize > self.config.current_tx_version {
            return Err(TransactionValidatorError::InvalidTransactionVersion(
                version,
                format!(
                    "Maximal valid version is {}.",
                    self.config.current_tx_version
                ),
            )
            .into());
        }
        if version_as_usize > self.config.max_allowed_tx_version {
            return Err(TransactionValidatorError::InvalidTransactionVersion(
                version,
                format!(
                    "Maximal supported version is {}.",
                    self.config.max_allowed_tx_version
                ),
            )
            .into());
        }

        // TODO(Arni, 1/4/2024): Validate fee and tx size.
        Ok(())
    }
}

fn get_tx_version(tx: &Transaction) -> TransactionVersion {
    match tx {
        Transaction::Declare(tx) => tx.version(),
        Transaction::DeployAccount(tx) => tx.version(),
        Transaction::Invoke(tx) => tx.version(),
        Transaction::Deploy(_) => TransactionVersion::ZERO,
        Transaction::L1Handler(_) => TransactionVersion::ZERO,
    }
}
