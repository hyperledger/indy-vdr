use super::storage::OrderedHashMap;
use super::CacheStrategy;
use async_lock::Mutex;
use async_trait::async_trait;
use std::{collections::BTreeMap, fmt::Debug, hash::Hash, sync::Arc, time::SystemTime};

/// A simple struct to hold a value and the expiry offset
/// needed because items can be inserted with custom ttl values
/// that may need to be updated/reordered
#[derive(Clone, Serialize, Deserialize)]
pub struct TTLCacheItem<V> {
    value: V,
    expire_offset: u128,
}

/// A simple cache that uses timestamps to expire entries. Once the cache fills up, the oldest entry is evicted.
/// Also uses LRU to evict entries that have not been accessed recently.
/// Uses a hashmap for lookups and a BTreeMap for ordering by age
pub struct CacheStrategyTTL<K, V> {
    store: Arc<Mutex<OrderedHashMap<K, u128, TTLCacheItem<V>>>>,
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
        store_type: Option<OrderedHashMap<K, u128, TTLCacheItem<V>>>,
        create_time: Option<SystemTime>,
    ) -> Self {
        Self {
            store: Arc::new(Mutex::new(match store_type {
                Some(store) => store,
                None => OrderedHashMap::new(BTreeMap::new()),
            })),
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
impl<K: Send + Sync + 'static, V: Send + Sync + 'static> CacheStrategy<K, V>
    for Arc<dyn CacheStrategy<K, V>>
{
    async fn get(&self, key: &K) -> Option<V> {
        self.get(key).await
    }
    async fn remove(&mut self, key: &K) -> Option<V> {
        self.remove(key).await
    }
    async fn insert(&mut self, key: K, value: V, custom_exp_offset: Option<u128>) -> Option<V> {
        self.insert(key, value, custom_exp_offset).await
    }
}

#[async_trait]
impl<K: Hash + Eq + Send + Sync + 'static + Clone + Debug, V: Clone + Send + Sync + 'static>
    CacheStrategy<K, V> for CacheStrategyTTL<K, V>
{
    async fn get(&self, key: &K) -> Option<V> {
        let mut store_lock = self.store.lock().await;
        let current_time = SystemTime::now()
            .duration_since(self.create_time)
            .unwrap()
            .as_millis();
        let get_res = match store_lock.get(key) {
            Some((ts, v)) => {
                if current_time < *ts {
                    Some((*ts, v.clone()))
                } else {
                    store_lock.remove(key);
                    None
                }
            }
            None => None,
        };
        // update the timestamp if the entry is still valid
        if let Some((_, ref v)) = get_res {
            store_lock.re_order(key, current_time + v.expire_offset);
        }
        get_res.map(|(_, v)| v.value)
    }
    async fn remove(&mut self, key: &K) -> Option<V> {
        self.store.lock().await.remove(key).map(|(_, v)| v.value)
    }

    async fn insert(&mut self, key: K, value: V, custom_exp_offset: Option<u128>) -> Option<V> {
        let mut store_lock = self.store.lock().await;
        let current_ts = SystemTime::now()
            .duration_since(self.create_time)
            .unwrap()
            .as_millis();

        // remove expired entries
        while store_lock.len() > 0
            && store_lock
                .get_first_key_value()
                .map(|(_, ts, _)| ts.clone() < current_ts)
                .unwrap_or(false)
        {
            store_lock.remove_first();
        }

        // remove the oldest item if the cache is still full
        if store_lock.len() >= self.capacity && store_lock.get(&key).is_none() {
            // remove the oldest item
            let removal_key = store_lock.get_first_key_value().map(|(k, _, _)| k.clone());
            if let Some(removal_key) = removal_key {
                store_lock.remove(&removal_key);
            }
        };

        let exp_offset = custom_exp_offset.unwrap_or(self.expire_after);
        store_lock
            .insert(
                key,
                TTLCacheItem {
                    value: value,
                    expire_offset: exp_offset,
                },
                current_ts + exp_offset,
            )
            .map(|v| v.value)
    }
}

#[cfg(test)]
mod tests {

    use std::thread;

    use super::*;
    use crate::pool::cache::{storage::OrderedHashMap, Cache};
    use futures_executor::block_on;

    #[rstest]
    fn test_cache_ttl() {
        let cache = Cache::new(CacheStrategyTTL::new(2, 5, None, None), None);
        let cache_location = "test_fs_cache_ttl";
        let tree = sled::open(cache_location)
            .unwrap()
            .open_tree(cache_location)
            .unwrap();
        let storage: OrderedHashMap<String, u128, TTLCacheItem<String>> = OrderedHashMap::new(tree);
        let fs_cache = Cache::new(CacheStrategyTTL::new(2, 5, Some(storage), None), None);
        let caches = vec![cache, fs_cache];
        block_on(async {
            for cache in caches {
                cache
                    .insert("key".to_string(), "value".to_string(), None)
                    .await;
                assert_eq!(
                    cache.get(&"key".to_string()).await,
                    Some("value".to_string())
                );
                cache
                    .insert("key1".to_string(), "value1".to_string(), None)
                    .await;
                cache
                    .insert("key2".to_string(), "value2".to_string(), None)
                    .await;
                assert_eq!(cache.get(&"key".to_string()).await, None);
                cache
                    .insert("key3".to_string(), "value3".to_string(), None)
                    .await;
                cache.get(&"key2".to_string()).await;
                cache
                    .insert("key4".to_string(), "value4".to_string(), None)
                    .await;
                // key2 should not be evicted because of LRU
                assert_eq!(
                    cache.remove(&"key2".to_string()).await,
                    Some("value2".to_string())
                );
                // key3 should be evicted because it was bumped to back after key2 was accessed
                assert_eq!(cache.get(&"key3".to_string()).await, None);
                cache
                    .insert("key5".to_string(), "value5".to_string(), None)
                    .await;
                thread::sleep(std::time::Duration::from_millis(6));
                assert_eq!(cache.get(&"key5".to_string()).await, None);
                // test ttl config
                cache
                    .insert("key6".to_string(), "value6".to_string(), Some(1))
                    .await;
                cache
                    .insert("key7".to_string(), "value7".to_string(), None)
                    .await;
                // wait until value6 expires
                thread::sleep(std::time::Duration::from_millis(1));
                assert_eq!(cache.get(&"key6".to_string()).await, None);
                assert_eq!(
                    cache.get(&"key7".to_string()).await,
                    Some("value7".to_string())
                );
            }
            std::fs::remove_dir_all(cache_location).unwrap();
        });
    }
}
