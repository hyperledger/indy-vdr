use std::{collections::HashMap, hash::Hash};

use futures_util::{future, Future};

use super::RequestResultMeta;

pub trait Cacheable<K, V> {
    fn get_cached_request(&self, key: K) -> impl Future<Output = Option<(V, RequestResultMeta)>>;

    fn cache_request(
        &mut self,
        key: K,
        result: V,
        meta: RequestResultMeta,
    ) -> impl Future<Output = Option<(V, RequestResultMeta)>>;
}

pub struct MemCache<K, V> {
    cache: HashMap<K, (V, RequestResultMeta, u64)>,
}

impl<K, V> MemCache<K, V> {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

impl<K: Hash + Eq, V: Clone> Cacheable<K, V> for MemCache<K, V> {
    fn get_cached_request(&self, key: K) -> impl Future<Output = Option<(V, RequestResultMeta)>> {
        future::ready(self.cache.get(&key).and_then(|(v, m, _)| Some((v.clone(), m.clone()))))
    }

    fn cache_request(
        &mut self,
        key: K,
        result: V,
        meta: RequestResultMeta,
    ) -> impl Future<Output = Option<(V, RequestResultMeta)>> {
        future::ready(self.cache.insert(key, (result, meta, 0)).and_then(|(v, m, _)| Some((v, m))))
    }
}
