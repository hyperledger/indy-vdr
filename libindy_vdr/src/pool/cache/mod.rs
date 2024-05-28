use std::{
    fmt::Display,
    sync::{Arc, RwLock},
};

pub mod storage;
pub mod strategy;

pub trait CacheStrategy<K, V>: Send + Sync + 'static {
    fn get(&self, key: &K) -> Option<V>;

    fn remove(&self, key: &K) -> Option<V>;

    fn insert(&self, key: K, value: V, custom_exp_offset: Option<u128>) -> Option<V>;
}

pub struct Cache<K: Display, V> {
    storage: Arc<RwLock<dyn CacheStrategy<String, V>>>,
    key_prefix: Option<K>,
}

impl<K: Display + 'static, V: 'static> Cache<K, V> {
    fn full_key(&self, key: &K) -> String {
        match &self.key_prefix {
            Some(prefix) => format!("{}{}", prefix, key),
            None => key.to_string(),
        }
    }

    pub fn new(storage: impl CacheStrategy<String, V>, key_prefix: Option<K>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
            key_prefix,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let full_key = self.full_key(key);
        if let Ok(storage) = self.storage.read() {
            return storage.get(&full_key);
        }
        None
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        let full_key = self.full_key(key);
        if let Ok(storage) = self.storage.write() {
            return storage.remove(&full_key);
        }
        None
    }

    pub fn insert(&self, key: K, value: V, custom_exp_offset: Option<u128>) -> Option<V> {
        let full_key = self.full_key(&key);
        if let Ok(storage) = self.storage.write() {
            return storage.insert(full_key, value, custom_exp_offset);
        }
        None
    }
}

// need to implement Clone manually because Mutex<dyn CacheStrategy> doesn't implement Clone
impl<K: Display + Clone, V> Clone for Cache<K, V> {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            key_prefix: self.key_prefix.clone(),
        }
    }
}
