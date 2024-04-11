use thiserror::Error;

#[derive(Debug, Error)]
pub enum MempoolError {
    #[error("Duplicate transaction")]
    DuplicateTransaction,
}
