use async_trait::async_trait;
use std::{
    collections::HashMap,
    hash::Hash,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::sync::RwLock;

// cant use async traits yet because not object safe
#[async_trait]
pub trait CacheStorage<K, V>: Send + Sync + 'static {
    async fn get(&self, key: &K) -> Option<(V, u64)>;

    async fn remove(&mut self, key: &K) -> Option<(V, u64)>;

    async fn insert(&mut self, key: K, value: V, expiration: u64) -> Option<(V, u64)>;
}

pub struct Cache<K, V> {
    storage: Arc<RwLock<dyn CacheStorage<K, V>>>,
    expiration_offset: u64,
}

impl<K: 'static, V: 'static> Cache<K, V> {
    pub fn new(storage: impl CacheStorage<K, V>, expiration_offset: u64) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
            expiration_offset,
        }
    }
    pub async fn get(&self, key: &K) -> Option<V> {
        match self.storage.read().await.get(key).await {
            Some((item, expiry)) => {
                if expiry > 0
                    && expiry
                        < SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                {
                    None
                } else {
                    Some(item)
                }
            }
            None => None,
        }
    }
    pub async fn remove(&mut self, key: &K) -> Option<V> {
        match self.storage.write().await.remove(key).await {
            Some(item) => Some(item.0),
            None => None,
        }
    }
    pub async fn insert(&mut self, key: K, value: V) -> Option<V> {
        let exp_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + self.expiration_offset;
        match self
            .storage
            .write()
            .await
            .insert(key, value, exp_timestamp)
            .await
        {
            Some(item) => Some(item.0),
            None => None,
        }
    }
}

// need to implement Clone manually because RwLock<dyn CacheStorage> doesn't implement Clone
impl<K, V> Clone for Cache<K, V> {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            expiration_offset: self.expiration_offset,
        }
    }
}

pub struct MemCacheStorage<K, V> {
    cache: HashMap<K, (V, u64)>,
}

impl<K, V> MemCacheStorage<K, V> {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}
#[async_trait]
impl<K: Hash + Eq + Send + Sync + 'static, V: Clone + Send + Sync + 'static> CacheStorage<K, V>
    for MemCacheStorage<K, V>
{
    async fn get(&self, key: &K) -> Option<(V, u64)> {
        self.cache.get(key).map(|(v, e)| (v.clone(), *e))
    }
    async fn remove(&mut self, key: &K) -> Option<(V, u64)> {
        self.cache.remove(key)
    }
    async fn insert(&mut self, key: K, value: V, expiration: u64) -> Option<(V, u64)> {
        self.cache
            .insert(key, (value, expiration))
            .map(|(v, e)| (v.clone(), e))
    }
}

#[cfg(test)]
mod tests {

    use futures_executor::block_on;

    #[rstest]
    fn test_cache() {
        use super::*;
        use std::{thread, time::Duration};

        let mut cache = Cache::new(MemCacheStorage::new(), 1);
        block_on(async {
            cache.insert("key".to_string(), "value".to_string()).await;
            assert_eq!(
                cache.get(&"key".to_string()).await,
                Some("value".to_string())
            );
            thread::sleep(Duration::from_secs(2));
            assert_eq!(cache.get(&"key".to_string()).await, None);
            assert_eq!(
                cache.remove(&"key".to_string()).await,
                Some("value".to_string())
            );
        });
    }
}
