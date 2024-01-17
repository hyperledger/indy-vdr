use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::sync::RwLock;

use super::RequestResultMeta;

// cant use async traits yet because not object safe
pub trait CacheStorage<K, V>: Send + Sync + 'static {
    fn get(&self, key: &K) -> Option<(V, u64)>;

    fn insert(&mut self, key: K, value: V, expiration: u64) -> Option<(V, u64)>;
}

pub struct Cache<K, V> {
    storage: Arc<RwLock<dyn CacheStorage<K, V>>>,
    expiration_offset: u64,
}

impl<K:'static, V: 'static> Cache<K, V> {
    pub fn new(storage: impl CacheStorage<K, V>, expiration_offset: u64) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
            expiration_offset,
        }
    }
    pub async fn get(&self, key: &K) -> Option<V> {
        match self.storage.read().await.get(key) {
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
    pub async fn insert(&mut self, key: K, value: V) -> Option<V> {
        let exp_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + self.expiration_offset;
        match self.storage.write().await.insert(key, value, exp_timestamp) {
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

pub struct MemCacheStorage {
    cache: HashMap<String, (String, RequestResultMeta, u64)>,
}

impl MemCacheStorage {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

impl CacheStorage<String, (String, RequestResultMeta)> for MemCacheStorage {
    fn get(&self, key: &String) -> Option<((String, RequestResultMeta), u64)> {
        self.cache
            .get(key)
            .map(|(v, m, e)| ((v.clone(), m.clone()), *e))
    }

    fn insert(
        &mut self,
        key: String,
        value: (String, RequestResultMeta),
        expiration: u64,
    ) -> Option<((String, RequestResultMeta), u64)> {
        self.cache
            .insert(key, (value.0, value.1, expiration))
            .map(|(v, m, e)| ((v.clone(), m.clone()), e))
    }
}

// pub trait Cacheable<K, V>: Clone + Send + Sync + 'static {
//     fn get_cached_request(&self, key: K) -> impl Future<Output = Option<(V, RequestResultMeta)>>;

//     fn cache_request(
//         &mut self,
//         key: K,
//         result: V,
//         meta: RequestResultMeta,
//     ) -> impl Future<Output = Option<(V, RequestResultMeta)>>;
// }

// pub struct MemCache<K, V> {
//     cache: Arc<RwLock<HashMap<K, (V, RequestResultMeta, u64)>>>,
// }

// impl<K, V> MemCache<K, V> {
//     pub fn new() -> Self {
//         Self {
//             cache: Arc::new(RwLock::new(HashMap::new())),
//         }
//     }
// }

// need to implement Clone manually because RwLock<HashMap> doesn't implement Clone
// impl<K, V> Clone for MemCache<K, V> {
//     fn clone(&self) -> Self {
//         Self {
//             cache: self.cache.clone(),
//         }
//     }
// }

// impl<K: Hash + Eq + Send + Sync + 'static, V: Clone + Send + Sync + 'static> Cacheable<K, V>
//     for MemCache<K, V>
// {
//     fn get_cached_request(&self, key: K) -> impl Future<Output = Option<(V, RequestResultMeta)>> {
//         future::ready(match self.cache.read() {
//             Ok(cache) => cache
//                 .get(&key)
//                 .and_then(|(v, m, _)| Some((v.clone(), m.clone()))),
//             Err(err) => {
//                 warn!("Error reading cache: {}", err);
//                 None
//             }
//         })
//     }

//     fn cache_request(
//         &mut self,
//         key: K,
//         result: V,
//         meta: RequestResultMeta,
//     ) -> impl Future<Output = Option<(V, RequestResultMeta)>> {
//         future::ready(match self.cache.write() {
//             Ok(mut cache) => cache
//                 .insert(key, (result, meta, 0))
//                 .and_then(|(v, m, _)| Some((v, m))),
//             Err(err) => {
//                 warn!("Error writing cache: {}", err);
//                 None
//             }
//         })
//     }
// }
