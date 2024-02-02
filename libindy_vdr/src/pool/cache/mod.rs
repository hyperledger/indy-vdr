use async_lock::RwLock;
use async_trait::async_trait;
use std::{fmt::Display, sync::Arc};

pub mod storage;
pub mod strategy;

#[async_trait]
pub trait CacheStrategy<K, V>: Send + Sync + 'static {
    async fn get(&self, key: &K) -> Option<V>;

    async fn remove(&mut self, key: &K) -> Option<V>;

    async fn insert(&mut self, key: K, value: V, custom_exp_offset: Option<u128>) -> Option<V>;
}

pub struct Cache<K, V> {
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

    pub async fn get(&self, key: &K) -> Option<V> {
        let full_key = self.full_key(key);
        self.storage.read().await.get(&full_key).await
    }

    pub async fn remove(&self, key: &K) -> Option<V> {
        let full_key = self.full_key(key);
        self.storage.write().await.remove(&full_key).await
    }

    pub async fn insert(&self, key: K, value: V, custom_exp_offset: Option<u128>) -> Option<V> {
        let full_key = self.full_key(&key);
        self.storage
            .write()
            .await
            .insert(full_key, value, custom_exp_offset)
            .await
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
