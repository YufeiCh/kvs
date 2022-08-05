use failure::Fail;
use sled::Error;
use std::io;
use std::string::FromUtf8Error;

/// Error type for KvStore error
#[derive(Fail, Debug)]
pub enum KvsError {
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
    /// sled error
    #[fail(display = "sled error: {}", _0)]
    Sled(#[cause] sled::Error),
    /// utf-8 tran error
    #[fail(display = "UTF-8 error: {}", _0)]
    Utf8(#[cause] FromUtf8Error),
    /// String error
    #[fail(display = "{}", _0)]
    StringError(String),
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

impl From<sled::Error> for KvsError {
    fn from(error: sled::Error) -> Self {
        KvsError::Sled(error)
    }
}

impl From<FromUtf8Error> for KvsError {
    fn from(error: FromUtf8Error) -> Self {
        KvsError::Utf8(error)
    }
}

/// Result type for KvStore
pub type Result<T> = std::result::Result<T, KvsError>;
