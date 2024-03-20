use starknet_api::transaction::{
    DeclareTransaction, DeployAccountTransaction, Fee, InvokeTransaction, ResourceBoundsMapping,
    Transaction, TransactionVersion,
};

use thiserror::Error;

#[cfg(test)]
#[path = "transaction_validator_test.rs"]
mod transaction_validator_test;

#[derive(Debug, Error)]
#[cfg_attr(test, derive(PartialEq))]
pub enum TransactionValidatorError {
    #[error("This transaction type is not supported by the mempool")]
    TransactionTypeNotSupported,
    #[error("Transactions of version {0:?} are not valid. {1}")]
    InvalidTransactionVersion(TransactionVersion, String),
    #[error("Blocked transaction version {0:?}. {1}")]
    BlockedTransactionVersion(TransactionVersion, String),
    #[error("Transaction must commit to pay a positive amount on fee.")]
    ZeroFee,
}

pub type TransactionValidatorResult<T> = Result<T, TransactionValidatorError>;

#[derive(Default)]
pub struct TransactionValidatorConfig {
    pub block_declare_cairo1: bool,
    pub block_declare_cairo0: bool,

    pub min_allowed_tx_version: TransactionVersion,
    pub max_allowed_tx_version: TransactionVersion,

    pub enforce_fee: bool,
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

        // TODO(Arni, 1/4/2024): Validate tx size.
        self.validate_fee(&tx)?;

        Ok(())
    }

    fn validate_fee(&self, tx: &Transaction) -> TransactionValidatorResult<()> {
        if !self.config.enforce_fee {
            return Ok(());
        }

        if tx.max_fee()? == Fee(0) {
            return Err(TransactionValidatorError::ZeroFee);
        }

        Ok(())
    }
}

trait MaxFeeExt {
    fn max_fee(&self) -> TransactionValidatorResult<Fee>;
}

impl MaxFeeExt for Transaction {
    fn max_fee(&self) -> TransactionValidatorResult<Fee> {
        match self {
            Transaction::Declare(declare_tx) => declare_tx.max_fee(),
            Transaction::DeployAccount(deploy_account_tx) => deploy_account_tx.max_fee(),
            Transaction::Invoke(invoke_tx) => invoke_tx.max_fee(),
            Transaction::Deploy(_) => Err(TransactionValidatorError::TransactionTypeNotSupported),
            Transaction::L1Handler(_) => {
                Err(TransactionValidatorError::TransactionTypeNotSupported)
            }
        }
    }
}

impl MaxFeeExt for DeclareTransaction {
    fn max_fee(&self) -> TransactionValidatorResult<Fee> {
        match self {
            starknet_api::transaction::DeclareTransaction::V0(_)
            | starknet_api::transaction::DeclareTransaction::V1(_)
            | starknet_api::transaction::DeclareTransaction::V2(_) => {
                Err(TransactionValidatorError::TransactionTypeNotSupported)
                // This code will return the proper max_fee.
                // _deprecated_declare_tx.max_fee
            }
            starknet_api::transaction::DeclareTransaction::V3(declare_v3_tx) => {
                declare_v3_tx.resource_bounds.max_fee()
            }
        }
    }
}

impl MaxFeeExt for DeployAccountTransaction {
    fn max_fee(&self) -> TransactionValidatorResult<Fee> {
        match self {
            starknet_api::transaction::DeployAccountTransaction::V1(_) => {
                Err(TransactionValidatorError::TransactionTypeNotSupported)
                // This code will return the proper max_fee.
                // _deprecated_deploy_account_tx.max_fee
            }
            starknet_api::transaction::DeployAccountTransaction::V3(deploy_account_v3_tx) => {
                deploy_account_v3_tx.resource_bounds.max_fee()
            }
        }
    }
}

impl MaxFeeExt for InvokeTransaction {
    fn max_fee(&self) -> TransactionValidatorResult<Fee> {
        match self {
            starknet_api::transaction::InvokeTransaction::V0(_)
            | starknet_api::transaction::InvokeTransaction::V1(_) => {
                Err(TransactionValidatorError::TransactionTypeNotSupported)
                // This code will return the proper max_fee.
                // _deprecated_invoke_tx.max_fee
            }
            starknet_api::transaction::InvokeTransaction::V3(invoke_v3_tx) => {
                invoke_v3_tx.resource_bounds.max_fee()
            }
        }
    }
}

impl MaxFeeExt for ResourceBoundsMapping {
    fn max_fee(&self) -> TransactionValidatorResult<Fee> {
        let l1_bounds = self
            .0
            .get(&starknet_api::transaction::Resource::L1Gas)
            .expect("resource bounds should contain L1Gas.");
        let max_amount = l1_bounds.max_amount;
        let max_price_per_unit = l1_bounds.max_price_per_unit;
        Ok(Fee(u128::from(max_amount) * max_price_per_unit))
    }
}

trait TransactionVersionExt {
    fn version(&self) -> TransactionVersion;
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
