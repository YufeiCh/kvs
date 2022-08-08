#![deny(missing_docs)]
//! A simple key-value store

pub use client::KvsClient;
pub use engines::{KvStore, KvsEngine, SledKvsEngine};
pub use error::{KvsError, Result};
pub use server::KvsServer;
pub use threadPool::{NaiveThreadPool, ThreadPool};

mod client;
mod common;
mod engines;
mod error;
mod server;
mod threadPool;
