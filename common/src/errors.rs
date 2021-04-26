use thiserror::Error as ThisError;

/// Errors that broker encounters
#[derive(Debug, ThisError)]
pub enum Error {
    /// Data passed was null
    #[error("Host Received Null Pointer")]
    NullPtr,
    /// Bincode Error
    #[error("{0:?}")]
    Bincode(#[from] bincode::Error),
    /// Custom Error
    #[error("{0}")]
    Custom(String),
}

impl Error {
    /// Create a custom error
    pub fn custom<T: Into<String>>(v: T) -> Self {
        Self::Custom(v.into())
    }
}