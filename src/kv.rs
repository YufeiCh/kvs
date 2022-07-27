use std::collections::HashMap;

use clap::Error;

/// The KvStore stores string key/value pairs
/// 
/// Example:
/// 
/// ```rust
/// # use kvs::KvStore;
/// let mut kvstore = KvStore::new();
/// kvstore.set("key1".to_owned(), "value1".to_owned());
/// let val = kvstore.get("key1".to_owned());
/// assert_eq!(val, Some("value1".to_owned()));
/// ```
pub struct KvStore {
    m : HashMap<String, String>,
}

impl KvStore {
    /// Create a new KvStore with HashMap
    pub fn new() -> KvStore {
        KvStore {
            m:HashMap::new(),
        }
    }

    /// set a key/value pair to the KvStore, when key is replicated, the pre-value is overwritten
    pub fn set(&mut self, key:String, value:String) -> Result<(), Error> {
        self.m.insert(key, value);
        Ok(())
    }

    /// remove a key from the KvStore
    pub fn remove(&mut self, key:String) {
        self.m.remove(&key);
    }

    /// get a value from the KvStore
    pub fn get(&self, key:String) -> Option<String> {
        self.m.get(&key).cloned()
    }

    /// new a KvStore with the filePath
    pub fn open(_:&std::path::Path) -> Result<KvStore, Error> {
        Ok(KvStore::new())
    }
}

impl Default for KvStore {
    fn default() -> Self {
        Self::new()
    }
}