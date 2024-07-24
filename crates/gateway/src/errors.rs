use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use blockifier::state::errors::StateError;
use enum_assoc::Assoc;
use serde::Serialize;
use serde_json::{Error as SerdeError, Value};
use starknet_api::block::GasPrice;
use starknet_api::transaction::{Resource, ResourceBounds};
use strum::EnumIter;
use thiserror::Error;

use crate::compiler_version::{VersionId, VersionIdError};

pub type GatewayResult<T> = Result<T, GatewaySpecError>;

impl IntoResponse for GatewaySpecError {
    fn into_response(self) -> Response {
        let body = self.to_string();
        (StatusCode::from_u16(self.code()).expect("Expecting a valid error code"), body)
            .into_response()
    }
}

#[derive(Error, Debug, Assoc, Clone, EnumIter, Serialize, PartialEq)]
#[func(pub fn code(&self) -> u16)]
#[func(pub fn data(&self) -> Option<&str>)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GatewaySpecError {
    #[error("Class hash not found")]
    #[assoc(code = 28)]
    ClassHashNotFound,
    #[error("Class already declared")]
    #[assoc(code = 51)]
    ClassAlreadyDeclared,
    #[error("Invalid transaction nonce")]
    #[assoc(code = 52)]
    InvalidTransactionNonce,
    #[error("Max fee is smaller than the minimal transaction cost (validation plus fee transfer)")]
    #[assoc(code = 53)]
    InsufficientMaxFee,
    #[error("Account balance is smaller than the transaction's max_fee")]
    #[assoc(code = 54)]
    InsufficientAccountBalance,
    #[error("Account validation failed")]
    #[assoc(code = 55)]
    #[assoc(data = _0)]
    ValidationFailure(String),
    #[error("Compilation failed")]
    #[assoc(code = 56)]
    CompilationFailed,
    #[error("Contract class size it too large")]
    #[assoc(code = 57)]
    ContractClassSizeIsTooLarge,
    #[error("Sender address in not an account contract")]
    #[assoc(code = 58)]
    NonAccount,
    #[error("A transaction with the same hash already exists in the mempool")]
    #[assoc(code = 59)]
    DuplicateTx,
    #[error("the compiled class hash did not match the one supplied in the transaction")]
    #[assoc(code = 60)]
    CompiledClassHashMismatch,
    #[error("the transaction version is not supported")]
    #[assoc(code = 61)]
    UnsupportedTxVersion,
    #[error("the contract class version is not supported")]
    #[assoc(code = 62)]
    UnsupportedContractClassVersion,
    #[error("An unexpected error occurred")]
    #[assoc(code = 63)]
    #[assoc(data = _0)]
    UnexpectedError(String),
}

#[derive(Debug, Error)]
#[cfg_attr(test, derive(PartialEq))]
pub enum StatelessTransactionValidatorError {
    #[error("Expected a positive amount of {resource:?}. Got {resource_bounds:?}.")]
    ZeroResourceBounds { resource: Resource, resource_bounds: ResourceBounds },
    #[error(
        "Calldata length exceeded maximum: length {calldata_length}
        (allowed length: {max_calldata_length})."
    )]
    CalldataTooLong { calldata_length: usize, max_calldata_length: usize },
    #[error(
        "Signature length exceeded maximum: length {signature_length}
        (allowed length: {max_signature_length})."
    )]
    SignatureTooLong { signature_length: usize, max_signature_length: usize },
    #[error(transparent)]
    InvalidSierraVersion(#[from] VersionIdError),
    #[error(
        "Sierra versions older than {min_version} or newer than {max_version} are not supported. \
         The Sierra version of the declared contract is {version}."
    )]
    UnsupportedSierraVersion { version: VersionId, min_version: VersionId, max_version: VersionId },
    #[error(
        "Cannot declare contract class with bytecode size of {bytecode_size}; max allowed size: \
         {max_bytecode_size}."
    )]
    BytecodeSizeTooLarge { bytecode_size: usize, max_bytecode_size: usize },
    #[error(
        "Cannot declare contract class with size of {contract_class_object_size}; max allowed \
         size: {max_contract_class_object_size}."
    )]
    ContractClassObjectSizeTooLarge {
        contract_class_object_size: usize,
        max_contract_class_object_size: usize,
    },
    #[error("Entry points must be unique and sorted.")]
    EntryPointsNotUniquelySorted,
}

impl From<StatelessTransactionValidatorError> for GatewaySpecError {
    fn from(e: StatelessTransactionValidatorError) -> Self {
        match e {
            StatelessTransactionValidatorError::ZeroResourceBounds { .. } => {
                GatewaySpecError::ValidationFailure(e.to_string())
            }
            StatelessTransactionValidatorError::CalldataTooLong { .. } => {
                GatewaySpecError::ValidationFailure(e.to_string())
            }
            StatelessTransactionValidatorError::SignatureTooLong { .. } => {
                GatewaySpecError::ValidationFailure(e.to_string())
            }
            StatelessTransactionValidatorError::InvalidSierraVersion(..) => {
                GatewaySpecError::ValidationFailure(e.to_string())
            }
            StatelessTransactionValidatorError::UnsupportedSierraVersion { .. } => {
                GatewaySpecError::UnsupportedContractClassVersion
            }
            StatelessTransactionValidatorError::BytecodeSizeTooLarge { .. } => {
                GatewaySpecError::ContractClassSizeIsTooLarge
            }
            StatelessTransactionValidatorError::ContractClassObjectSizeTooLarge { .. } => {
                GatewaySpecError::ContractClassSizeIsTooLarge
            }
            StatelessTransactionValidatorError::EntryPointsNotUniquelySorted => {
                GatewaySpecError::ValidationFailure(e.to_string())
            }
        }
    }
}

pub type StatelessTransactionValidatorResult<T> = Result<T, StatelessTransactionValidatorError>;

pub type StatefulTransactionValidatorResult<T> = Result<T, GatewaySpecError>;

/// Errors originating from `[`Gateway::run`]` command, to be handled by infrastructure code.
#[derive(Debug, Error)]
pub enum GatewayRunError {
    #[error(transparent)]
    ServerStartupError(#[from] hyper::Error),
}

#[derive(Debug, Error)]
pub enum RPCStateReaderError {
    #[error("Block not found for request {0}")]
    BlockNotFound(Value),
    #[error("Class hash not found for request {0}")]
    ClassHashNotFound(Value),
    #[error("Failed to parse gas price {:?}", 0)]
    GasPriceParsingFailure(GasPrice),
    #[error("Contract address not found for request {0}")]
    ContractAddressNotFound(Value),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("RPC error: {0}")]
    RPCError(StatusCode),
    #[error("Unexpected error code: {0}")]
    UnexpectedErrorCode(u16),
}

pub type RPCStateReaderResult<T> = Result<T, RPCStateReaderError>;

impl From<RPCStateReaderError> for StateError {
    fn from(err: RPCStateReaderError) -> Self {
        match err {
            RPCStateReaderError::ClassHashNotFound(request) => {
                match serde_json::from_value(request["params"]["class_hash"].clone()) {
                    Ok(class_hash) => StateError::UndeclaredClassHash(class_hash),
                    Err(e) => serde_err_to_state_err(e),
                }
            }
            _ => StateError::StateReadError(err.to_string()),
        }
    }
}

// Converts a serde error to the error type of the state reader.
pub fn serde_err_to_state_err(err: SerdeError) -> StateError {
    StateError::StateReadError(format!("Failed to parse rpc result {:?}", err.to_string()))
}
