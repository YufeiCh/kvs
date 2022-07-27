use failure::Fail;
use std::{fmt, io};

/// Error type for KvStore error
#[derive(Fail, Debug)]
pub enum KvError{
    /// IoError
    IoError(io::Error),

}

impl fmt::Display for KvError {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        write!(f, "kv error")
    }
}

impl From<io::Error> for KvError {
    fn from(error: io::Error) -> Self {
        KvError::IoError(error)
    }
}

/// Result type for KvStore
pub type Result<T> = std::result::Result<T, KvError>;