use starknet_api::transaction::Transaction;
use starknet_api::transaction::TransactionVersion;

use thiserror::Error;

#[cfg(test)]
#[path = "transaction_validator_test.rs"]
mod transaction_validator_test;

#[derive(Debug, Error)]
#[cfg_attr(test, derive(PartialEq))]
pub enum TransactionValidatorError {
    #[error("Invalid transaction type")]
    InvalidTransactionType,
    #[error("Transactions of version {0:?} are not valid. {1}")]
    InvalidTransactionVersion(TransactionVersion, String),
    #[error("Blocked transaction version {0:?}. {1}")]
    BlockedTransactionVersion(TransactionVersion, String),
}

pub type TransactionValidatorResult<T> = Result<T, TransactionValidatorError>;

#[derive(Default)]
pub struct TransactionValidatorConfig {
    pub block_declare_cairo1: bool,
    pub block_declare_cairo0: bool,

    pub min_allowed_tx_version: TransactionVersion,
    pub max_allowed_tx_version: TransactionVersion,
}

pub struct TransactionValidator {
    pub config: TransactionValidatorConfig,
}

impl TransactionValidator {
    pub fn validate(&self, tx: Transaction) -> TransactionValidatorResult<()> {
        // Deploy transactions are deprecated.
        // L1Handler transactions are not supported in the gateway.
        if matches!(tx, Transaction::Deploy(_) | Transaction::L1Handler(_)) {
            return Err(TransactionValidatorError::InvalidTransactionType);
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
        let version = get_tx_version(&tx);
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
