use cairo_lang_starknet_classes::compiler_version::VersionId;
use starknet_api::external_transaction::{
    ExternalDeclareTransaction, ExternalDeployAccountTransaction, ExternalInvokeTransaction,
    ExternalTransaction, ResourceBoundsMapping,
};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::Resource;

use crate::config::StatelessTransactionValidatorConfig;
use crate::errors::{StatelessTransactionValidatorError, StatelessTransactionValidatorResult};

// TODO(Arni): Get from config.
pub const MAX_SIERRA_VERSION: VersionId = VersionId { major: 1, minor: 5, patch: 0 };
pub const MIN_SIERRA_VERSION: VersionId = VersionId { major: 1, minor: 1, patch: 0 };

#[cfg(test)]
#[path = "stateless_transaction_validator_test.rs"]
mod stateless_transaction_validator_test;

#[derive(Clone)]
pub struct StatelessTransactionValidator {
    pub config: StatelessTransactionValidatorConfig,
}

impl StatelessTransactionValidator {
    pub fn validate(&self, tx: &ExternalTransaction) -> StatelessTransactionValidatorResult<()> {
        // TODO(Arni, 1/5/2024): Add a mechanism that validate the sender address is not blocked.
        // TODO(Arni, 1/5/2024): Validate transaction version.

        self.validate_resource_bounds(tx)?;
        self.validate_tx_size(tx)?;

        if let ExternalTransaction::Declare(declare_tx) = tx {
            self.validate_declare_tx(declare_tx)?;
        }
        Ok(())
    }

    fn validate_resource_bounds(
        &self,
        tx: &ExternalTransaction,
    ) -> StatelessTransactionValidatorResult<()> {
        let resource_bounds_mapping = tx.resource_bounds();

        if self.config.validate_non_zero_l1_gas_fee {
            validate_resource_is_non_zero(resource_bounds_mapping, Resource::L1Gas)?;
        }
        if self.config.validate_non_zero_l2_gas_fee {
            validate_resource_is_non_zero(resource_bounds_mapping, Resource::L2Gas)?;
        }

        Ok(())
    }

    fn validate_tx_size(
        &self,
        tx: &ExternalTransaction,
    ) -> StatelessTransactionValidatorResult<()> {
        self.validate_tx_calldata_size(tx)?;
        self.validate_tx_signature_size(tx)?;

        Ok(())
    }

    fn validate_tx_calldata_size(
        &self,
        tx: &ExternalTransaction,
    ) -> StatelessTransactionValidatorResult<()> {
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
            return Err(StatelessTransactionValidatorError::CalldataTooLong {
                calldata_length,
                max_calldata_length: self.config.max_calldata_length,
            });
        }

        Ok(())
    }

    fn validate_tx_signature_size(
        &self,
        tx: &ExternalTransaction,
    ) -> StatelessTransactionValidatorResult<()> {
        let signature = tx.signature();

        let signature_length = signature.0.len();
        if signature_length > self.config.max_signature_length {
            return Err(StatelessTransactionValidatorError::SignatureTooLong {
                signature_length,
                max_signature_length: self.config.max_signature_length,
            });
        }

        Ok(())
    }

    fn validate_declare_tx(
        &self,
        declare_tx: &ExternalDeclareTransaction,
    ) -> StatelessTransactionValidatorResult<()> {
        let contract_class = match declare_tx {
            ExternalDeclareTransaction::V3(tx) => &tx.contract_class,
        };
        self.validate_sierra_version(&contract_class.sierra_program)?;
        self.validate_class_length(contract_class)
    }

    fn validate_sierra_version(
        &self,
        sierra_program: &[StarkFelt],
    ) -> StatelessTransactionValidatorResult<()> {
        let sierra_version = sierra_program_version_id(sierra_program)?;

        // Check that the version is not too old.
        if less_then(sierra_version, MIN_SIERRA_VERSION) {
            return Err(StatelessTransactionValidatorError::UnsupportedSierraVersion {
                version: sierra_version,
                min_version: MIN_SIERRA_VERSION,
                max_version: MAX_SIERRA_VERSION,
            });
        }
        // Check that the version is lower than the latest version allowing higher patch versions
        // (i.e. we ignore the Z part in a version X.Y.Z).
        let max_minor_sierra_version = VersionId { patch: 0, ..MAX_SIERRA_VERSION };
        let minor_sierra_version = VersionId { patch: 0, ..sierra_version };

        if less_then(max_minor_sierra_version, minor_sierra_version) {
            return Err(StatelessTransactionValidatorError::UnsupportedSierraVersion {
                version: sierra_version,
                min_version: MIN_SIERRA_VERSION,
                max_version: MAX_SIERRA_VERSION,
            });
        }

        Ok(())
    }

    fn validate_class_length(
        &self,
        contract_class: &starknet_api::external_transaction::ContractClass,
    ) -> StatelessTransactionValidatorResult<()> {
        let bytecode_size = contract_class.sierra_program.len();
        if bytecode_size > self.config.max_bytecode_size {
            return Err(StatelessTransactionValidatorError::BytecodeSizeTooLarge {
                bytecode_size,
                max_bytecode_size: self.config.max_bytecode_size,
            });
        }

        let contract_class_object_size = serde_json::to_string(&contract_class)
            .expect("Unexpected error serializing contract class.")
            .len();
        if contract_class_object_size > self.config.max_raw_class_size {
            return Err(StatelessTransactionValidatorError::ContractClassObjectSizeTooLarge {
                contract_class_object_size,
                max_contract_class_object_size: self.config.max_raw_class_size,
            });
        }

        Ok(())
    }
}

fn sierra_program_version_id(
    sierra_program: &[StarkFelt],
) -> StatelessTransactionValidatorResult<VersionId> {
    let length_of_version = sierra_program.len().min(3);
    let mut version = [StarkFelt::default(); 3];
    version[..length_of_version].copy_from_slice(&sierra_program[..length_of_version]);

    if length_of_version < 3 {
        return Err(StatelessTransactionValidatorError::InvalidSierraVersion { version });
    }

    let map_err = || StatelessTransactionValidatorError::InvalidSierraVersion { version };
    Ok(VersionId {
        major: sierra_program[0].try_into().map_err(|_| map_err())?,
        minor: sierra_program[1].try_into().map_err(|_| map_err())?,
        patch: sierra_program[2].try_into().map_err(|_| map_err())?,
    })
}

fn less_then(lhs: VersionId, rhs: VersionId) -> bool {
    match lhs.major.cmp(&rhs.major) {
        std::cmp::Ordering::Less => true,
        std::cmp::Ordering::Greater => false,
        std::cmp::Ordering::Equal => match lhs.minor.cmp(&rhs.minor) {
            std::cmp::Ordering::Less => true,
            std::cmp::Ordering::Greater => false,
            std::cmp::Ordering::Equal => lhs.patch < rhs.patch,
        },
    }
}

fn validate_resource_is_non_zero(
    resource_bounds_mapping: &ResourceBoundsMapping,
    resource: Resource,
) -> StatelessTransactionValidatorResult<()> {
    let resource_bounds = match resource {
        Resource::L1Gas => resource_bounds_mapping.l1_gas,
        Resource::L2Gas => resource_bounds_mapping.l2_gas,
    };
    if resource_bounds.max_amount == 0 || resource_bounds.max_price_per_unit == 0 {
        return Err(StatelessTransactionValidatorError::ZeroResourceBounds {
            resource,
            resource_bounds,
        });
    }

    Ok(())
}
