use crate::Result;
/// Trait for a key value store engine
pub trait KvsEngine: Clone + Send + 'static {
    /// set a key/value pair to the KvStore, when key is replicated, the pre-value is overwritten
    fn set(&self, key: String, value: String) -> Result<()>;

    /// get a value from the KvStore
    fn get(&self, key: String) -> Result<Option<String>>;

    /// remove a key from the KvStore
    fn remove(&self, key: String) -> Result<()>;
}

mod kv;
mod sled;

pub use self::kv::KvStore;
pub use self::sled::SledKvsEngine;
