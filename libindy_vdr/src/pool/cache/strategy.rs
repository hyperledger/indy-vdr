use super::storage::OrderedHashMap;
use super::CacheStrategy;
use async_lock::Mutex;
use async_trait::async_trait;
use std::{collections::BTreeMap, fmt::Debug, hash::Hash, sync::Arc, time::SystemTime};
/// A simple cache that uses timestamps to expire entries. Once the cache fills up, the oldest entry is evicted.
/// Uses a hashmap for lookups and a BTreeMap for ordering by age
pub struct CacheStrategyTTL<K, V> {
    store: OrderedHashMap<K, u128, V>,
    capacity: usize,
    create_time: SystemTime,
    expire_after: u128,
}

impl<K: Eq + Hash + Clone + Send + Sync + 'static, V: Clone + Send + Sync + 'static>
    CacheStrategyTTL<K, V>
{
    /// Create a new cache with the given capacity and expiration time in milliseconds
    /// If store_type is None, the cache will use an in-memory hashmap and BTreeMap
    /// cache_time is used as a starting point to generate timestamps if it is None, the cache will use the UNIX_EPOCH as the cache start time
    pub fn new(
        capacity: usize,
        expire_after: u128,
        store_type: Option<OrderedHashMap<K, u128, V>>,
        create_time: Option<SystemTime>,
    ) -> Self {
        Self {
            store: match store_type {
                Some(store) => store,
                None => OrderedHashMap::new(BTreeMap::new()),
            },
            capacity,
            create_time: match create_time {
                Some(time) => time,
                None => SystemTime::UNIX_EPOCH,
            },
            expire_after,
        }
    }
}

#[async_trait]
impl<K: Hash + Eq + Send + Sync + 'static + Clone + Debug, V: Clone + Send + Sync + 'static>
    CacheStrategy<K, V> for CacheStrategyTTL<K, V>
{
    async fn get(&self, key: &K) -> Option<V> {
        match self.store.get(key) {
            Some((ts, v)) => {
                let current_time = SystemTime::now()
                    .duration_since(self.create_time)
                    .unwrap()
                    .as_millis();
                if current_time < *ts {
                    Some(v.clone())
                } else {
                    None
                }
            }
            None => None,
        }
    }
    async fn remove(&mut self, key: &K) -> Option<V> {
        self.store.remove(key).map(|(_, v)| v)
    }
    async fn insert(&mut self, key: K, value: V) -> Option<V> {
        let current_ts = SystemTime::now()
            .duration_since(self.create_time)
            .unwrap()
            .as_millis();

        // remove expired entries
        while self.store.len() > 0
            && self
                .store
                .get_first_key_value()
                .map(|(_, ts, _)| ts.clone() < current_ts)
                .unwrap_or(false)
        {
            self.store.remove_first();
        }

        // remove the oldest item if the cache is still full
        if self.store.len() >= self.capacity && self.store.get(&key).is_none() {
            // remove the oldest item
            let removal_key = self.store.get_first_key_value().map(|(k, _, _)| k.clone());
            if let Some(removal_key) = removal_key {
                self.store.remove(&removal_key);
            }
        };

        let exp_offset = self.expire_after;
        self.store.insert(key, value, current_ts + exp_offset)
    }
}

/// A simple LRU cache. Once the cache fills up, the least recently used entry is evicted.
/// Uses a hashmap for lookups and a BTreeMap for ordering by least recently used
pub struct CacheStrategyLRU<K, V> {
    // The store is wrapped in an arc and a mutex so that get() can be immutable
    store: Arc<Mutex<OrderedHashMap<K, u128, V>>>,
    capacity: usize,
}

impl<K: Eq + Hash + Clone + Send + Sync + 'static, V: Clone + Send + Sync + 'static>
    CacheStrategyLRU<K, V>
{
    pub fn new(capacity: usize, store_type: Option<OrderedHashMap<K, u128, V>>) -> Self {
        Self {
            store: Arc::new(Mutex::new(match store_type {
                Some(store) => store,
                None => OrderedHashMap::new(BTreeMap::new()),
            })),
            capacity,
        }
    }
}
#[async_trait]
impl<K: Hash + Eq + Send + Sync + 'static + Clone, V: Clone + Send + Sync + 'static>
    CacheStrategy<K, V> for CacheStrategyLRU<K, V>
{
    async fn get(&self, key: &K) -> Option<V> {
        // move the key to the end of the LRU index
        // this is O(log(n))
        let mut store_lock = self.store.lock().await;
        let highest_lru = store_lock
            .get_last_key_value()
            .map(|(_, ts, _)| ts + 1)
            .unwrap_or(0);
        store_lock.re_order(key, highest_lru);
        store_lock.get(key).map(|(_, v)| v.clone())
    }
    async fn remove(&mut self, key: &K) -> Option<V> {
        let mut store_lock = self.store.lock().await;
        store_lock.remove(key).map(|(_, v)| v)
    }
    async fn insert(&mut self, key: K, value: V) -> Option<V> {
        // this will be O(log(n)) in all cases when cache is at capacity since we need to fetch the first and last element from the btree
        let mut store_lock = self.store.lock().await;
        let highest_lru = store_lock
            .get_last_key_value()
            .map(|(_, ts, _)| ts + 1)
            .unwrap_or(0);

        if store_lock.len() >= self.capacity && store_lock.get(&key).is_none() {
            // remove the LRU item
            let lru_key = store_lock
                .get_first_key_value()
                .map(|(k, _, _)| k.clone())
                .unwrap();
            store_lock.remove(&lru_key);
        };

        store_lock.insert(key, value, highest_lru)
    }
}

#[cfg(test)]
mod tests {

    use std::thread;

    use super::*;
    use crate::pool::cache::{storage::OrderedHashMap, Cache};
    use futures_executor::block_on;

    #[rstest]
    fn test_cache_lru() {
        let cache = Cache::new(CacheStrategyLRU::new(2, None));
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
    fn test_fs_cache_lru() {
        let cache_location = "test_fs_cache_lru";
        let tree = sled::open(cache_location)
            .unwrap()
            .open_tree(cache_location)
            .unwrap();
        let storage: OrderedHashMap<String, u128, String> = OrderedHashMap::new(tree);
        let cache = Cache::new(CacheStrategyLRU::new(2, Some(storage)));
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

            // cleanup
            std::fs::remove_dir_all(cache_location).unwrap();
        });
    }

    #[rstest]
    fn test_cache_ttl() {
        let cache = Cache::new(CacheStrategyTTL::new(2, 5, None, None));
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
            thread::sleep(std::time::Duration::from_millis(6));
            assert_eq!(cache.get(&"key5".to_string()).await, None);
        });
    }

    #[rstest]
    fn test_fs_cache_ttl() {
        let cache_location = "test_fs_cache_ttl";
        let tree = sled::open(cache_location)
            .unwrap()
            .open_tree(cache_location)
            .unwrap();
        let storage: OrderedHashMap<String, u128, String> = OrderedHashMap::new(tree);
        let cache = Cache::new(CacheStrategyTTL::new(2, 5, Some(storage), None));
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
            thread::sleep(std::time::Duration::from_millis(6));
            assert_eq!(cache.get(&"key5".to_string()).await, None);

            // cleanup
            std::fs::remove_dir_all(cache_location).unwrap();
        });
    }
}
