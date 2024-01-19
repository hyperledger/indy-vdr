use async_trait::async_trait;
use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    ops::DerefMut,
    sync::Arc,
    time::SystemTime,
};

use async_lock::Mutex;

#[async_trait]
pub trait CacheStorage<K, V>: Send + Sync + 'static {
    async fn get(&self, key: &K) -> Option<V>;

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

/// A simple in-memory cache that uses timestamps to expire entries. Once the cache fills up, the oldest entry is evicted.
/// Uses a hashmap for lookups and a BTreeMap for ordering by age
pub struct MemCacheStorageTTL<K, V> {
    store: (HashMap<K, (V, u128)>, BTreeMap<u128, Vec<K>>),
    capacity: usize,
    startup_time: SystemTime,
}

impl<K, V> MemCacheStorageTTL<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            store: (HashMap::new(), BTreeMap::new()),
            capacity,
            startup_time: SystemTime::now(),
        }
    }
}

#[async_trait]
impl<K: Hash + Eq + Send + Sync + 'static + Clone, V: Clone + Send + Sync + 'static>
    CacheStorage<K, V> for MemCacheStorageTTL<K, V>
{
    async fn get(&self, key: &K) -> Option<V> {
        let (cache_lookup, _) = &self.store;
        match cache_lookup.get(key) {
            Some((v, _)) => Some(v.clone()),
            None => None,
        }
    }
    async fn remove(&mut self, key: &K) -> Option<V> {
        let (cache_lookup, cache_order) = &mut self.store;
        let ttl_val = cache_lookup.remove(key);
        match ttl_val {
            Some((v, ts)) => {
                let val = cache_order.get_mut(&ts).unwrap();
                if val.len() <= 1 {
                    cache_order.remove(&ts);
                } else {
                    val.retain(|k| k != key);
                }
                Some(v)
            }
            None => None,
        }
    }
    async fn insert(&mut self, key: K, value: V) -> Option<V> {
        let (cache_lookup, cache_order) = &mut self.store;
        let ts = SystemTime::now()
            .duration_since(self.startup_time)
            .unwrap()
            .as_millis();
        // only remove the oldest item if the cache is full and the key is not already in the cache
        if cache_lookup.len() >= self.capacity && cache_lookup.get(&key).is_none() {
            // remove the oldest item
            let (oldest_ts_ref, _) = cache_order.first_key_value().unwrap();
            let oldest_ts = *oldest_ts_ref;
            let oldest_keys = cache_order.get_mut(&oldest_ts).unwrap();
            let removal_key = oldest_keys.first().and_then(|k| Some(k.clone()));
            if oldest_keys.len() <= 1 {
                // remove the whole array since it's the last entry
                cache_order.remove(&oldest_ts);
            } else {
                oldest_keys.swap_remove(0);
            }
            cache_lookup.remove(&key);
            if let Some(removal_key) = removal_key {
                cache_lookup.remove(&removal_key);
            }
        };

        // if value is overwritten when inserting a new key, we need to remove the old key from the order index
        cache_order.entry(ts).or_insert(vec![]).push(key.clone());
        match cache_lookup.insert(key.clone(), (value.clone(), ts)) {
            Some((v, ts)) => {
                if let Some(ord_keys) = cache_order.get_mut(&ts) {
                    if ord_keys.len() <= 1 {
                        cache_order.remove(&ts);
                    } else {
                        ord_keys.retain(|k| k != &key);
                    }
                }
                Some(v)
            }
            None => None,
        }
    }
}

/// A simple in-memory LRU cache. Once the cache fills up, the least recently used entry is evicted.
/// Uses a hashmap for lookups and a BTreeMap for ordering by least recently used
pub struct MemCacheStorageLRU<K, V> {
    // The store is wrapped in an arc and a mutex so that get() can be immutable
    store: Arc<Mutex<(HashMap<K, (V, u64)>, BTreeMap<u64, K>)>>,
    capacity: usize,
}

impl<K, V> MemCacheStorageLRU<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            store: Arc::new(Mutex::new((HashMap::new(), BTreeMap::new()))),
            capacity,
        }
    }
}
#[async_trait]
impl<K: Hash + Eq + Send + Sync + 'static + Clone, V: Clone + Send + Sync + 'static>
    CacheStorage<K, V> for MemCacheStorageLRU<K, V>
{
    async fn get(&self, key: &K) -> Option<V> {
        // move the key to the end of the LRU index
        // this is O(log(n)) in the worst case, but in the average case it's close to O(1)
        let mut store_lock = self.store.lock().await;
        let (cache_lookup, cache_order) = store_lock.deref_mut();
        let highest_lru = cache_order
            .last_key_value()
            .map(|(hts, _)| hts + 1)
            .unwrap_or(0);
        match cache_lookup.get_mut(key) {
            Some((v, ts)) => {
                cache_order.remove(ts).unwrap();
                cache_order.entry(highest_lru).or_insert(key.clone());
                *ts = highest_lru;
                Some(v.clone())
            }
            None => None,
        }
    }
    async fn remove(&mut self, key: &K) -> Option<V> {
        let mut store_lock = self.store.lock().await;
        let (cache_lookup, cache_order) = store_lock.deref_mut();
        let lru_val = cache_lookup.remove(key);
        match lru_val {
            Some((v, ts)) => {
                cache_order.remove(&ts);
                Some(v)
            }
            None => None,
        }
    }
    async fn insert(&mut self, key: K, value: V) -> Option<V> {
        // this will be O(log(n)) in all cases when cache is at capacity since we need to fetch the first and last element from the btree
        let mut store_lock = self.store.lock().await;
        let (cache_lookup, cache_order) = store_lock.deref_mut();
        let highest_lru = cache_order
            .last_key_value()
            .map(|(ts, _)| ts + 1)
            .unwrap_or(0);
        if cache_lookup.len() >= self.capacity && cache_lookup.get(&key).is_none() {
            // remove the LRU item
            let (lru_ts, lru_key) = match cache_order.first_key_value() {
                Some((ts, key)) => (*ts, key.clone()),
                None => return None,
            };
            cache_lookup.remove(&lru_key);
            cache_order.remove(&lru_ts);
        };

        // if value is overwritten when inserting a new key, we need to remove the old key from the order index
        cache_order.insert(highest_lru, key.clone());
        match cache_lookup.insert(key.clone(), (value.clone(), highest_lru)) {
            Some((v, ts)) => {
                cache_order.remove(&ts);
                Some(v)
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {

    use std::thread;

    use futures_executor::block_on;

    #[rstest]
    fn test_cache_lru() {
        use super::*;

        let mut cache = Cache::new(MemCacheStorageLRU::new(2));
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

    #[rstest]
    fn test_cache_ttl() {
        use super::*;

        let mut cache = Cache::new(MemCacheStorageTTL::new(2));
        block_on(async {
            cache.insert("key".to_string(), "value".to_string()).await;
            thread::sleep(std::time::Duration::from_millis(1));
            assert_eq!(
                cache.get(&"key".to_string()).await,
                Some("value".to_string())
            );
            cache.insert("key1".to_string(), "value1".to_string()).await;
            thread::sleep(std::time::Duration::from_millis(1));
            cache.insert("key2".to_string(), "value2".to_string()).await;
            assert_eq!(cache.get(&"key".to_string()).await, None);
            thread::sleep(std::time::Duration::from_millis(1));
            cache.insert("key3".to_string(), "value3".to_string()).await;
            cache.get(&"key2".to_string()).await;
            cache.insert("key4".to_string(), "value4".to_string()).await;
            // key2 should be evicted
            assert_eq!(cache.remove(&"key2".to_string()).await, None);
            assert_eq!(
                cache.remove(&"key3".to_string()).await,
                Some("value3".to_string())
            );
            cache.insert("key5".to_string(), "value5".to_string()).await;
        });
    }
}
