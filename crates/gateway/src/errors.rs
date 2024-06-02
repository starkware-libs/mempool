use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use blockifier::blockifier::stateful_validator::StatefulValidatorError;
use blockifier::state::errors::StateError;
use blockifier::transaction::errors::TransactionExecutionError;
use cairo_lang_starknet_classes::compiler_version::VersionId;
use starknet_api::block::BlockNumber;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{Resource, ResourceBounds};
use starknet_api::StarknetApiError;
use thiserror::Error;
use tokio::task::JoinError;

/// Errors directed towards the end-user, as a result of gateway requests.
#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("Internal server error: {0}")]
    InternalServerError(#[from] JoinError),
    #[error("Error sending message: {0}")]
    MessageSendError(String),
    #[error(transparent)]
    StatefulTransactionValidatorError(#[from] StatefulTransactionValidatorError),
    #[error(transparent)]
    StatelessTransactionValidatorError(#[from] StatelessTransactionValidatorError),
}

impl IntoResponse for GatewayError {
    // TODO(Arni, 1/5/2024): Be more fine tuned about the error response. Not all Gateway errors
    // are internal server errors.
    fn into_response(self) -> Response {
        let body = self.to_string();
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
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
    DeclareTransactionError(#[from] DeclareTransactionError),
}

#[derive(Debug, Error)]
#[cfg_attr(test, derive(PartialEq))]
pub enum DeclareTransactionError {
    #[error("Length of Sierra program: {length}. Sierra program: {program_prefix:?}")]
    SierraProgramTooShort { length: usize, program_prefix: [StarkFelt; 2] },
    #[error("Invalid character in Sierra version: {version:?}.")]
    InvalidSierraVersion { version: [StarkFelt; 3] },
    // The checks for this are probably already covered in the compiler's repo: See
    // `StarknetSierraCompilationError::UnsupportedSierraVersion`.
    #[error("Sierra version {version} is below the minimum version {min_version}.")]
    VersionBelowMinimum { version: VersionId, min_version: VersionId },
    #[error("Sierra version {version} is above the maximum version {max_version}.")]
    VersionAboveMaximum { version: VersionId, max_version: VersionId },
    #[error(
        "Declared contract class {bytecode_language} bytecode size is {bytecode_size}. It must be \
         less then {max_bytecode_size}."
    )]
    BytecodeSizeTooLarge {
        bytecode_language: String,
        bytecode_size: usize,
        max_bytecode_size: usize,
    },
    #[error(
        "Declared contract class {bytecode_language} size is {contract_class_object_size}. It \
         must be less then {max_contract_class_object_size}."
    )]
    ContractClassObjectSizeTooLarge {
        bytecode_language: String,
        contract_class_object_size: usize,
        max_contract_class_object_size: usize,
    },
}

pub type StatelessTransactionValidatorResult<T> = Result<T, StatelessTransactionValidatorError>;

#[derive(Debug, Error)]
pub enum StatefulTransactionValidatorError {
    #[error("Block number {block_number:?} is out of range.")]
    OutOfRangeBlockNumber { block_number: BlockNumber },
    #[error(transparent)]
    StarknetApiError(#[from] StarknetApiError),
    #[error(transparent)]
    StateError(#[from] StateError),
    #[error(transparent)]
    StatefulValidatorError(#[from] StatefulValidatorError),
    #[error(transparent)]
    TransactionExecutionError(#[from] TransactionExecutionError),
}

pub type StatefulTransactionValidatorResult<T> = Result<T, StatefulTransactionValidatorError>;

/// Errors originating from `[`Gateway::run`]` command, to be handled by infrastructure code.
#[derive(Debug, Error)]
pub enum GatewayRunError {
    #[error(transparent)]
    ServerStartupError(#[from] hyper::Error),
}
