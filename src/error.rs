use failure::Fail;
use std::io;

/// Error type for KvStore error
#[derive(Fail, Debug)]
pub enum KvsError{
    /// Io Error
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
    /// Json Error
    #[fail(display = "{}", _0)]
    Serde(#[cause] serde_json::Error),
    ///key not found
    #[fail(display = "Key not found")]
    KeyNotFound,
    /// invalid cmd
    #[fail(display = "Unexpected command type")]
    UnexpectedCommandType,
}


impl From<io::Error> for KvsError {
    fn from(error: io::Error) -> Self {
        KvsError::Io(error)
    }
}

impl From<serde_json::Error> for KvsError {
    fn from(error: serde_json::Error) -> Self {
        KvsError::Serde(error)
    }
}

/// Result type for KvStore
pub type Result<T> = std::result::Result<T, KvsError>;