use thiserror::Error as ThisError;

/// Errors that broker encounters
#[derive(Debug, ThisError)]
pub enum Error {
    /// Data passed to host was null
    #[error("Host Received Null Pointer")]
    NullPtr,
    /// Dissection wasn't run
    #[error("Not Run")]
    NotRun,
    /// Failure acquiring lock in hostcall
    #[error("Lock")]
    Lock,
    /// Custom Error
    #[error("{0}")]
    Custom(String),
    /// Invalid offset for WasmPtr
    #[error("Invalid offset")]
    InvalidOffset,
    /// Common Library Error
    #[error("{0:?}")]
    Common(#[from] common::Error),
    /// Bincode Error
    #[error("{0:?}")]
    Bincode(#[from] bincode::Error),
    /// Export
    #[error("{0:?}")]
    Export(#[from] wasmer::ExportError),
    /// Instantiation
    #[error("{0:?}")]
    Instantiation(#[from] wasmer::InstantiationError),
    /// Runtime
    #[error("{0:?}")]
    Memory(#[from] wasmer::MemoryError),
    /// Runtime
    #[error("{0:?}")]
    Runtime(#[from] wasmer::RuntimeError),
    /// State
    #[error("{0:?}")]
    State(#[from] wasmer_wasi::WasiStateCreationError),
    /// Wasi Support Error
    #[error("Wasi: {0:?}")]
    WasiSupport(String),
    /// String conversion error
    #[error("{0:?}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

impl Error {
    /// Create a custom error
    pub fn custom<T: Into<String>>(v: T) -> Self {
        Self::Custom(v.into())
    }
}