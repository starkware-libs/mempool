use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilationUtilError {
    #[error("Starknet Sierra compilation error: {0}")]
    CompilationError(String),
}
