use async_lock::RwLock;
use async_trait::async_trait;
use std::sync::Arc;

pub mod storage;
pub mod strategy;

#[async_trait]
pub trait CacheStrategy<K, V>: Send + Sync + 'static {
    async fn get(&self, key: &K) -> Option<V>;

    async fn remove(&mut self, key: &K) -> Option<V>;

    async fn insert(&mut self, key: K, value: V) -> Option<V>;
}

pub struct Cache<K, V> {
    storage: Arc<RwLock<dyn CacheStrategy<K, V>>>,
}

impl<K: 'static, V: 'static> Cache<K, V> {
    pub fn new(storage: impl CacheStrategy<K, V>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
        }
    }
    pub async fn get(&self, key: &K) -> Option<V> {
        self.storage.read().await.get(key).await
    }
    pub async fn remove(&self, key: &K) -> Option<V> {
        self.storage.write().await.remove(key).await
    }
    pub async fn insert(&self, key: K, value: V) -> Option<V> {
        self.storage.write().await.insert(key, value).await
    }
}

// need to implement Clone manually because Mutex<dyn CacheStrategy> doesn't implement Clone
impl<K, V> Clone for Cache<K, V> {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
        }
    }
}
