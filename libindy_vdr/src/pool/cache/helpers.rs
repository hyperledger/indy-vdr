use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};
/// A hashmap that also maintains a BTreeMap of keys ordered by a given value
/// This is useful for structures that need fast O(1) lookups, but also need to evict the oldest or least recently used entries
pub(crate) struct OrderedHashMap<K, O, V>((HashMap<K, (O, V)>, BTreeMap<O, Vec<K>>));

impl<K, O, V> OrderedHashMap<K, O, V> {
    pub(crate) fn new() -> Self {
        Self((HashMap::new(), BTreeMap::new()))
    }
}

impl<K: Hash + Eq + Clone, O: Ord + Copy, V> OrderedHashMap<K, O, V> {
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
        selector: Box<dyn Fn(&BTreeMap<O, Vec<K>>) -> Option<(&O, &Vec<K>)>>,
    ) -> Option<(&K, &O, &V)> {
        let (lookup, ordered_lookup) = &self.0;
        selector(ordered_lookup).and_then(|(_, keys)| {
            keys.first()
                .and_then(|key| lookup.get(key).and_then(|(o, v)| Some((key, o, v))))
        })
    }
    /// gets the entry with the lowest order value
    pub fn get_first_key_value(&self) -> Option<(&K, &O, &V)> {
        self.get_key_value(Box::new(|ordered_lookup| ordered_lookup.first_key_value()))
    }
    /// gets the entry with the highest order value
    pub fn get_last_key_value(&self) -> Option<(&K, &O, &V)> {
        self.get_key_value(Box::new(|ordered_lookup| ordered_lookup.last_key_value()))
    }
    /// re-orders the entry with the given key
    pub fn re_order(&mut self, key: &K, new_order: O) {
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
    pub fn insert(&mut self, key: K, value: V, order: O) -> Option<V> {
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
    pub fn remove(&mut self, key: &K) -> Option<(O, V)> {
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
