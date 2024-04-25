use starknet_api::StarknetApiError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MempoolError {
    #[error("Duplicate transaction")]
    DuplicateTransaction,
    #[error(transparent)]
    StarknetApiError(#[from] StarknetApiError),
}
