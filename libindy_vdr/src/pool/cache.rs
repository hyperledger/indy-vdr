use async_trait::async_trait;
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    hash::Hash,
    sync::Arc,
    time::SystemTime,
};

use async_lock::{Mutex, RwLock};

#[async_trait]
pub trait CacheStorage<K, V>: Send + Sync + 'static {
    async fn get(&self, key: &K) -> Option<V>;

    async fn remove(&mut self, key: &K) -> Option<V>;

    async fn insert(&mut self, key: K, value: V) -> Option<V>;
}

pub struct Cache<K, V> {
    storage: Arc<RwLock<dyn CacheStorage<K, V>>>,
}

impl<K: 'static, V: 'static> Cache<K, V> {
    pub fn new(storage: impl CacheStorage<K, V>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
        }
    }
    pub async fn get(&mut self, key: &K) -> Option<V> {
        self.storage.read().await.get(key).await
    }
    pub async fn remove(&mut self, key: &K) -> Option<V> {
        self.storage.write().await.remove(key).await
    }
    pub async fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.storage.write().await.insert(key, value).await
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

/// A hashmap that also maintains a BTreeMap of keys ordered by a given value
/// This is useful for structures that need fast O(1) lookups, but also need to evict the oldest or least recently used entries
struct OrderedHashMap<K, O, V>((HashMap<K, (O, V)>, BTreeMap<O, Vec<K>>));

impl<K, O, V> OrderedHashMap<K, O, V> {
    fn new() -> Self {
        Self((HashMap::new(), BTreeMap::new()))
    }
}

impl<K: Hash + Eq + Clone, O: Ord + Copy, V> OrderedHashMap<K, O, V> {
    fn len(&self) -> usize {
        let (lookup, _) = &self.0;
        lookup.len()
    }
    fn get(&self, key: &K) -> Option<&(O, V)> {
        let (lookup, _) = &self.0;
        lookup.get(key)
    }
    fn get_key_value(
        &self,
        selector: Box<dyn Fn(&BTreeMap<O, Vec<K>>) -> Option<(&O, &Vec<K>)>>,
    ) -> Option<(&K, &O, &V)> {
        let (lookup, ordered_lookup) = &self.0;
        selector(ordered_lookup).and_then(|(_, keys)| {
            keys.first()
                .and_then(|key| lookup.get(key).and_then(|(o, v)| Some((key, o, v))))
        })
    }
    /// gets the entry with the lowest order value
    fn get_first_key_value(&self) -> Option<(&K, &O, &V)> {
        self.get_key_value(Box::new(|ordered_lookup| ordered_lookup.first_key_value()))
    }
    /// gets the entry with the highest order value
    fn get_last_key_value(&self) -> Option<(&K, &O, &V)> {
        self.get_key_value(Box::new(|ordered_lookup| ordered_lookup.last_key_value()))
    }
    /// re-orders the entry with the given key
    fn re_order(&mut self, key: &K, new_order: O) {
        let (lookup, order_lookup) = &mut self.0;
        if let Some((old_order, _)) = lookup.get(key) {
            // remove entry in btree
            match order_lookup.get_mut(old_order) {
                Some(keys) => {
                    keys.retain(|k| k != key);
                    if keys.len() == 0 {
                        order_lookup.remove(old_order);
                    }
                }
                None => {}
            }
        }
        order_lookup
            .entry(new_order)
            .or_insert(vec![])
            .push(key.clone());
        lookup.get_mut(key).map(|(o, _)| *o = new_order);
    }
    /// inserts a new entry with the given key and value and order
    fn insert(&mut self, key: K, value: V, order: O) -> Option<V> {
        let (lookup, order_lookup) = &mut self.0;

        if let Some((old_order, _)) = lookup.get(&key) {
            // remove entry in btree
            match order_lookup.get_mut(old_order) {
                Some(keys) => {
                    keys.retain(|k| k != &key);
                    if keys.len() == 0 {
                        order_lookup.remove(old_order);
                    }
                }
                None => {}
            }
        }
        order_lookup
            .entry(order)
            .or_insert(vec![])
            .push(key.clone());
        lookup
            .insert(key, (order, value))
            .and_then(|(_, v)| Some(v))
    }
    /// removes the entry with the given key
    fn remove(&mut self, key: &K) -> Option<(O, V)> {
        let (lookup, order_lookup) = &mut self.0;
        lookup.remove(key).and_then(|(order, v)| {
            match order_lookup.get_mut(&order) {
                Some(keys) => {
                    keys.retain(|k| k != key);
                    if keys.len() == 0 {
                        order_lookup.remove(&order);
                    }
                }
                None => {}
            }
            Some((order, v))
        })
    }
    /// removes the entry with the lowest order value
    fn remove_first(&mut self) -> Option<(K, O, V)> {
        let first_key = self.get_first_key_value().map(|(k, _, _)| k.clone());
        if let Some(first_key) = first_key {
            self.remove(&first_key)
                .map(|(order, v)| (first_key, order, v))
        } else {
            None
        }
    }
}

/// A simple in-memory cache that uses timestamps to expire entries. Once the cache fills up, the oldest entry is evicted.
/// Uses a hashmap for lookups and a BTreeMap for ordering by age
pub struct MemCacheStorageTTL<K, V> {
    store: OrderedHashMap<K, u128, V>,
    capacity: usize,
    startup_time: SystemTime,
    expire_after: u128,
}

impl<K, V> MemCacheStorageTTL<K, V> {
    /// Create a new cache with the given capacity and expiration time in milliseconds
    pub fn new(capacity: usize, expire_after: u128) -> Self {
        Self {
            store: OrderedHashMap::new(),
            capacity,
            startup_time: SystemTime::now(),
            expire_after,
        }
    }
}

#[async_trait]
impl<K: Hash + Eq + Send + Sync + 'static + Clone + Debug, V: Clone + Send + Sync + 'static>
    CacheStorage<K, V> for MemCacheStorageTTL<K, V>
{
    async fn get(&self, key: &K) -> Option<V> {
        match self.store.get(key) {
            Some((ts, v)) => {
                let current_time = SystemTime::now()
                    .duration_since(self.startup_time)
                    .unwrap()
                    .as_millis();
                if current_time < ts + self.expire_after {
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
            .duration_since(self.startup_time)
            .unwrap()
            .as_millis();

        // remove expired entries
        let exp_offset = self.expire_after;
        while self.store.len() > 0
            && self
                .store
                .get_first_key_value()
                .map(|(_, ts, _)| ts + exp_offset < current_ts)
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

        self.store.insert(key, value, current_ts)
    }
}

/// A simple in-memory LRU cache. Once the cache fills up, the least recently used entry is evicted.
/// Uses a hashmap for lookups and a BTreeMap for ordering by least recently used
pub struct MemCacheStorageLRU<K, V> {
    // The store is wrapped in an arc and a mutex so that get() can be immutable
    store: Arc<Mutex<OrderedHashMap<K, u64, V>>>,
    capacity: usize,
}

impl<K, V> MemCacheStorageLRU<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            store: Arc::new(Mutex::new(OrderedHashMap::new())),
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

        let mut cache = Cache::new(MemCacheStorageTTL::new(2, 5));
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
}
