use cairo_lang_starknet_classes::compiler_version::VersionId;
use starknet_api::external_transaction::{
    ExternalDeclareTransaction, ExternalDeclareTransactionV3, ExternalDeployAccountTransaction,
    ExternalInvokeTransaction, ExternalTransaction, ResourceBoundsMapping,
};
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::Resource;

use crate::config::StatelessTransactionValidatorConfig;
use crate::errors::{
    DeclareTransactionError, StatelessTransactionValidatorError,
    StatelessTransactionValidatorResult,
};

// TODO: Get from config. Validate the config - make sure Sierra version is:
// cairo_lang_starknet_classes::compiler_version::current_sierra_version_id.
pub const SIERRA_VERSION: VersionId = VersionId { major: 1, minor: 5, patch: 0 };
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

        if let ExternalTransaction::Declare(ExternalDeclareTransaction::V3(declare_tx)) = tx {
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
        tx: &ExternalDeclareTransactionV3,
    ) -> StatelessTransactionValidatorResult<()> {
        let sierra_program = &tx.contract_class.sierra_program;
        self.validate_sierra_version(sierra_program)?;
        self.validate_class_length(&tx.contract_class)?;

        Ok(())
    }

    fn validate_sierra_version(
        &self,
        sierra_program: &[StarkFelt],
    ) -> StatelessTransactionValidatorResult<()> {
        let sierra_version = sierra_version_from_sierra_program(sierra_program)?;

        // Check that the version is not too old.
        if less_then_version_id(sierra_version, MIN_SIERRA_VERSION) {
            return Err(StatelessTransactionValidatorError::DeclareTransactionError(
                DeclareTransactionError::VersionBelowMinimum {
                    version: sierra_version,
                    min_version: MIN_SIERRA_VERSION,
                },
            ));
        }
        // Check that the version is lower than the latest version allowing higher patch versions
        // (i.e. we ignore the Z part in a version X.Y.Z).
        let latest_minor_sierra_version = VersionId { patch: 0, ..SIERRA_VERSION };
        let minor_sierra_version = VersionId { patch: 0, ..sierra_version };

        if less_then_version_id(latest_minor_sierra_version, minor_sierra_version) {
            return Err(StatelessTransactionValidatorError::DeclareTransactionError(
                DeclareTransactionError::VersionAboveMaximum {
                    version: sierra_version,
                    max_version: SIERRA_VERSION,
                },
            ));
        }

        Ok(())
    }

    fn validate_class_length(
        &self,
        contract_class: &starknet_api::external_transaction::ContractClass,
    ) -> StatelessTransactionValidatorResult<()> {
        let bytecode_size = contract_class.sierra_program.len();
        let serialized_class = serde_json::to_string(&contract_class)
            .expect("Unexpected error serializing contract class.");
        let raw_class_size = serialized_class.len();

        Ok(validate_class_size(
            "Sierra".to_string(),
            bytecode_size,
            self.config.max_bytecode_size,
            raw_class_size,
            self.config.max_raw_class_size,
        )?)
    }
}

// Utilities.

fn validate_class_size(
    bytecode_language: String,
    bytecode_size: usize,
    max_bytecode_size: usize,
    raw_class_size: usize,
    max_raw_class_size: usize,
) -> Result<(), DeclareTransactionError> {
    if bytecode_size > max_bytecode_size {
        return Err(DeclareTransactionError::BytecodeSizeTooLarge {
            bytecode_language,
            bytecode_size,
            max_bytecode_size,
        });
    }

    if raw_class_size > max_raw_class_size {
        return Err(DeclareTransactionError::ContractClassObjectSizeTooLarge {
            bytecode_language,
            contract_class_object_size: raw_class_size,
            max_contract_class_object_size: max_raw_class_size,
        });
    }

    Ok(())
}

fn sierra_version_from_sierra_program(
    sierra_program: &[StarkFelt],
) -> Result<VersionId, DeclareTransactionError> {
    let len_of_sierra_program = sierra_program.len();
    if len_of_sierra_program < 3 {
        let mut sierra_version = [StarkFelt::default(); 2];
        sierra_version[..len_of_sierra_program]
            .copy_from_slice(&sierra_program[..len_of_sierra_program]);

        return Err(DeclareTransactionError::SierraProgramTooShort {
            length: len_of_sierra_program,
            program_prefix: sierra_version,
        });
    }

    let mut version_as_felts = [StarkFelt::default(); 3];
    version_as_felts.copy_from_slice(&sierra_program[..3]);
    let map_err = || DeclareTransactionError::InvalidSierraVersion { version: version_as_felts };
    Ok(VersionId {
        major: sierra_program[0].try_into().map_err(|_| map_err())?,
        minor: sierra_program[1].try_into().map_err(|_| map_err())?,
        patch: sierra_program[2].try_into().map_err(|_| map_err())?,
    })
}

fn less_then_version_id(lhs_version_id: VersionId, rhs_version_id: VersionId) -> bool {
    match lhs_version_id.major.cmp(&rhs_version_id.major) {
        std::cmp::Ordering::Less => true,
        std::cmp::Ordering::Greater => false,
        std::cmp::Ordering::Equal => match lhs_version_id.minor.cmp(&rhs_version_id.minor) {
            std::cmp::Ordering::Less => true,
            std::cmp::Ordering::Greater => false,
            std::cmp::Ordering::Equal => lhs_version_id.patch < rhs_version_id.patch,
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
