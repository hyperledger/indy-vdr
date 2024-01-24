use serde::{de::DeserializeOwned, Serialize};
use sled::{self, Tree};
use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

pub trait OrderedStore<O, V>: Send + Sync {
    fn len(&self) -> usize;
    fn first_key_value(&self) -> Option<(O, V)>;
    fn last_key_value(&self) -> Option<(O, V)>;
    fn get(&self, key: &O) -> Option<V>;
    fn insert(&mut self, key: O, value: V) -> Option<V>;
    fn remove(&mut self, key: &O) -> Option<V>;
    fn entries(&self) -> Box<dyn Iterator<Item = (O, V)> + '_>;
}
impl<V: Serialize + DeserializeOwned> OrderedStore<u128, V> for Tree {
    fn len(&self) -> usize {
        Tree::len(self)
    }
    fn first_key_value(&self) -> Option<(u128, V)> {
        match self.first() {
            Ok(Some((k, v))) => serde_json::from_slice(v.as_ref()).ok().map(|v| {
                (
                    u128::from_be_bytes(k.as_ref().try_into().unwrap_or([0; 16])),
                    v,
                )
            }),
            _ => None,
        }
    }
    fn last_key_value(&self) -> Option<(u128, V)> {
        match self.last() {
            Ok(Some((k, v))) => serde_json::from_slice(v.as_ref()).ok().map(|v| {
                (
                    u128::from_be_bytes(k.as_ref().try_into().unwrap_or([0; 16])),
                    v,
                )
            }),
            _ => None,
        }
    }
    fn get(&self, key: &u128) -> Option<V> {
        match Tree::get(self, key.to_be_bytes()).map(|v| v) {
            Ok(Some(v)) => serde_json::from_slice(v.as_ref()).ok(),
            _ => None,
        }
    }
    fn insert(&mut self, key: u128, value: V) -> Option<V> {
        match Tree::insert(self, key.to_be_bytes(), serde_json::to_vec(&value).unwrap()) {
            Ok(Some(v)) => serde_json::from_slice(v.as_ref()).ok(),
            _ => None,
        }
    }
    fn remove(&mut self, key: &u128) -> Option<V> {
        match Tree::remove(self, key.to_be_bytes()).map(|v| v) {
            Ok(Some(v)) => serde_json::from_slice(&v).ok(),
            _ => None,
        }
    }
    fn entries(&self) -> Box<dyn Iterator<Item = (u128, V)>> {
        Box::new(self.iter().filter_map(|r| {
            r.ok().and_then(|(k, v)| {
                serde_json::from_slice(v.as_ref())
                    .ok()
                    .map(|v| (u128::from_be_bytes(k.as_ref().try_into().unwrap()), v))
            })
        }))
    }
}
impl<O: Ord + Copy + Send + Sync, V: Clone + Send + Sync> OrderedStore<O, V> for BTreeMap<O, V> {
    fn len(&self) -> usize {
        BTreeMap::len(self)
    }
    fn first_key_value(&self) -> Option<(O, V)> {
        BTreeMap::first_key_value(self).map(|(o, v)| (*o, v.clone()))
    }
    fn last_key_value(&self) -> Option<(O, V)> {
        BTreeMap::last_key_value(self).map(|(o, v)| (*o, v.clone()))
    }
    fn get(&self, key: &O) -> Option<V> {
        BTreeMap::get(self, key).map(|v| v.clone())
    }
    fn insert(&mut self, key: O, value: V) -> Option<V> {
        BTreeMap::insert(self, key, value)
    }
    fn remove(&mut self, key: &O) -> Option<V> {
        BTreeMap::remove(self, key)
    }
    fn entries(&self) -> Box<dyn Iterator<Item = (O, V)> + '_> {
        Box::new(self.iter().map(|(o, v)| (o.clone(), v.clone())))
    }
}
/// A hashmap that also maintains a BTreeMap of keys ordered by a given value
/// This is useful for structures that need fast O(1) lookups, but also need to evict the oldest or least recently used entries
/// The Ordered Store must contain both the keys and values for persistence
pub struct OrderedHashMap<K, O, V>(
    (
        HashMap<K, (O, V)>,
        Box<dyn OrderedStore<O, Vec<(K, V)>> + Send + Sync>,
    ),
);

impl<K: Eq + Hash + Clone + Send + Sync, O: Ord + Clone + Send + Sync, V: Clone>
    OrderedHashMap<K, O, V>
{
    pub fn new(order: impl OrderedStore<O, Vec<(K, V)>> + 'static) -> Self {
        let ordered_data = Box::new(order);
        let mut keyed_data = HashMap::new();
        // ordered data may be from the FS, so we need to rebuild the keyed data
        ordered_data.entries().for_each(|(order, keys)| {
            keys.iter().for_each(|(k, v)| {
                keyed_data.insert(k.clone(), (order.clone(), v.clone()));
            })
        });
        Self((keyed_data, ordered_data))
    }
}

impl<K: Hash + Eq + Clone, O: Ord + Clone, V: Clone> OrderedHashMap<K, O, V> {
    pub fn len(&self) -> usize {
        let (lookup, _) = &self.0;
        lookup.len()
    }
    pub fn get(&self, key: &K) -> Option<&(O, V)> {
        let (lookup, _) = &self.0;
        lookup.get(key)
    }
    fn get_key_value(
        &self,
        selector: Box<
            dyn Fn(&Box<dyn OrderedStore<O, Vec<(K, V)>> + Send + Sync>) -> Option<(O, Vec<K>)>,
        >,
    ) -> Option<(K, O, V)> {
        let (lookup, ordered_lookup) = &self.0;
        selector(ordered_lookup).and_then(|(_, keys)| {
            keys.first().and_then(|key| {
                lookup
                    .get(key)
                    .and_then(|(o, v)| Some((key.clone(), o.clone(), v.clone())))
            })
        })
    }
    /// gets the entry with the lowest order value
    pub fn get_first_key_value(&self) -> Option<(K, O, V)> {
        self.get_key_value(Box::new(|ordered_lookup| {
            ordered_lookup.first_key_value().and_then(|(order, keys)| {
                Some((order.clone(), keys.into_iter().map(|(k, _)| k).collect()))
            })
        }))
    }
    /// gets the entry with the highest order value
    pub fn get_last_key_value(&self) -> Option<(K, O, V)> {
        self.get_key_value(Box::new(|ordered_lookup| {
            ordered_lookup.last_key_value().and_then(|(order, keys)| {
                Some((order.clone(), keys.into_iter().map(|(k, _)| k).collect()))
            })
        }))
    }
    /// re-orders the entry with the given new order
    pub fn re_order(&mut self, key: &K, new_order: O) {
        if let Some((_, value)) = self.remove(key) {
            self.insert(key.clone(), value, new_order);
        }
    }
    /// inserts a new entry with the given key and value and order
    pub fn insert(&mut self, key: K, value: V, order: O) -> Option<V> {
        let (lookup, order_lookup) = &mut self.0;

        if let Some((old_order, _)) = lookup.get(&key) {
            // if entry already exists, remove it from the btree
            if let Some(mut keys) = order_lookup.remove(old_order) {
                keys.retain(|k| k.0 != key);
                // insert modified keys back into btree
                if !keys.is_empty() {
                    order_lookup.insert(old_order.clone(), keys);
                }
            }
        }
        let keys = match order_lookup.remove(&order) {
            Some(mut ks) => {
                ks.push((key.clone(), value.clone()));
                ks
            }
            None => vec![(key.clone(), value.clone())],
        };
        order_lookup.insert(order.clone(), keys);
        lookup
            .insert(key, (order, value))
            .and_then(|(_, v)| Some(v))
    }
    /// removes the entry with the given key
    pub fn remove(&mut self, key: &K) -> Option<(O, V)> {
        let (lookup, order_lookup) = &mut self.0;
        lookup.remove(key).and_then(|(order, v)| {
            match order_lookup.remove(&order) {
                Some(mut keys) => {
                    keys.retain(|k| k.0 != *key);
                    // insert remaining keys back in
                    if !keys.is_empty() {
                        order_lookup.insert(order.clone(), keys);
                    }
                }
                None => {}
            }
            Some((order, v))
        })
    }
    /// removes the entry with the lowest order value
    pub fn remove_first(&mut self) -> Option<(K, O, V)> {
        let first_key = self.get_first_key_value().map(|(k, _, _)| k.clone());
        if let Some(first_key) = first_key {
            self.remove(&first_key)
                .map(|(order, v)| (first_key, order, v))
        } else {
            None
        }
    }
}
