#![deny(missing_docs)]
//! A simple key-value store

pub use kv::KvStore;
pub use error::Result;
pub use error::KvError;

mod kv;
mod error;