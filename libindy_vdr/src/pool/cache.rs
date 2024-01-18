use async_trait::async_trait;
use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    sync::Arc,
};

use async_lock::Mutex;

#[async_trait]
pub trait CacheStorage<K, V>: Send + Sync + 'static {
    // Needs to be mutable bc some implementations may need to update the the LRU index of the cache
    async fn get(&mut self, key: &K) -> Option<V>;

    async fn remove(&mut self, key: &K) -> Option<V>;

    async fn insert(&mut self, key: K, value: V) -> Option<V>;
}

pub struct Cache<K, V> {
    storage: Arc<Mutex<dyn CacheStorage<K, V>>>,
}

impl<K: 'static, V: 'static> Cache<K, V> {
    pub fn new(storage: impl CacheStorage<K, V>) -> Self {
        Self {
            storage: Arc::new(Mutex::new(storage)),
        }
    }
    pub async fn get(&mut self, key: &K) -> Option<V> {
        self.storage.lock().await.get(key).await
    }
    pub async fn remove(&mut self, key: &K) -> Option<V> {
        self.storage.lock().await.remove(key).await
    }
    pub async fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.storage.lock().await.insert(key, value).await
    }
}

// need to implement Clone manually because Mutex<dyn CacheStorage> doesn't implement Clone
impl<K, V> Clone for Cache<K, V> {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
        }
    }
}

/// A simple in-memory LRU cache
/// Uses a hashmap for lookups and a BTreeMap for ordering by least recently used
pub struct MemCacheStorage<K, V> {
    cache_lookup: HashMap<K, (V, u64)>,
    cache_order: BTreeMap<u64, K>,
    capacity: usize,
}

impl<K, V> MemCacheStorage<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache_lookup: HashMap::new(),
            cache_order: BTreeMap::new(),
            capacity,
        }
    }
}
#[async_trait]
impl<K: Hash + Eq + Send + Sync + 'static + Clone, V: Clone + Send + Sync + 'static>
    CacheStorage<K, V> for MemCacheStorage<K, V>
{
    async fn get(&mut self, key: &K) -> Option<V> {
        // move the key to the end of the LRU index
        // this is O(log(n)) in the worst case, but in the average case it's close to O(1)
        match self.cache_lookup.get(key) {
            Some((v, ts)) => {
                self.cache_order.remove(ts).unwrap();
                self.cache_order.entry(ts + 1).or_insert(key.clone());
                Some(v.clone())
            }
            None => None,
        }
    }
    async fn remove(&mut self, key: &K) -> Option<V> {
        let lru_val = self.cache_lookup.remove(key);
        match lru_val {
            Some((v, ts)) => {
                self.cache_order.remove(&ts);
                Some(v)
            }
            None => None,
        }
    }
    async fn insert(&mut self, key: K, value: V) -> Option<V> {
        // this will be O(log(n)) in all cases when cache is at capacity since we need to fetch the first and last element from the btree
        let highest_lru = self
            .cache_order
            .last_key_value()
            .map(|(ts, _)| ts + 1)
            .unwrap_or(0);
        if self.cache_lookup.len() >= self.capacity {
            // remove the LRU item
            let (lru_ts, lru_key) = match self.cache_order.first_key_value() {
                Some((ts, key)) => (*ts, key.clone()),
                None => return None,
            };
            self.cache_lookup.remove(&lru_key);
            self.cache_order.remove(&lru_ts);
        };

        self.cache_order.insert(highest_lru, key.clone());
        match self
            .cache_lookup
            .insert(key.clone(), (value.clone(), highest_lru))
        {
            Some((v, _)) => Some(v),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {

    use futures_executor::block_on;

    #[rstest]
    fn test_cache() {
        use super::*;

        let mut cache = Cache::new(MemCacheStorage::new(2));
        block_on(async {
            cache.insert("key".to_string(), "value".to_string()).await;
            assert_eq!(
                cache.get(&"key".to_string()).await,
                Some("value".to_string())
            );
            cache.insert("key1".to_string(), "value1".to_string()).await;
            cache.insert("key2".to_string(), "value2".to_string()).await;
            assert_eq!(cache.get(&"key".to_string()).await, None);
            cache.insert("key3".to_string(), "value3".to_string()).await;
            cache.get(&"key2".to_string()).await; // move key2 to the end of the LRU index
            cache.insert("key4".to_string(), "value4".to_string()).await;
            // key3 should be evicted
            assert_eq!(
                cache.remove(&"key2".to_string()).await,
                Some("value2".to_string())
            );
            assert_eq!(cache.remove(&"key3".to_string()).await, None);
        });
    }
}
