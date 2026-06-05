use std::collections::VecDeque;
use std::hash::Hash;

use crate::HashMap;

/// Fixed-size least-recently-used cache.
///
/// The front of `priority` is the most recently used key and the back is the
/// least recently used key. Values are owned by the cache so callers can move
/// them back out with [`LruCache::take`] when needed.
pub struct LruCache<K, V, const SIZE: usize>
where
    K: Eq + Hash + Clone,
{
    cache: HashMap<K, V>,
    priority: VecDeque<K>,
}

impl<K, V, const SIZE: usize> Default for LruCache<K, V, SIZE>
where
    K: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self {
            cache: HashMap::default(),
            priority: VecDeque::default(),
        }
    }
}

impl<K, V, const SIZE: usize> LruCache<K, V, SIZE>
where
    K: Eq + Hash + Clone,
{
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.cache.contains_key(key)
    }

    fn remove_priority_key(&mut self, key: &K) -> bool {
        if let Some(index) = self.priority.iter().position(|k| k == key) {
            self.priority.remove(index);
            true
        } else {
            false
        }
    }

    fn mark_recent(&mut self, key: &K) {
        self.remove_priority_key(key);
        self.priority.push_front(key.clone());
    }

    /// Returns a reference to a cached value and marks it as recently used.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if !self.cache.contains_key(key) {
            return None;
        }

        self.mark_recent(key);
        self.cache.get(key)
    }

    /// Removes a value from the cache and returns ownership of it.
    pub fn take(&mut self, key: &K) -> Option<V> {
        let value = self.cache.remove(key);

        if value.is_some() {
            self.remove_priority_key(key);
        }

        value
    }

    /// Removes a value from the cache.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.take(key)
    }

    /// Inserts or replaces a value.
    ///
    /// Returns the evicted least-recently-used entry when inserting a new value
    /// would exceed the cache capacity. Replacing an existing value does not
    /// evict another entry.
    pub fn put(&mut self, key: K, value: V) -> Option<(K, V)> {
        if SIZE == 0 {
            return Some((key, value));
        }

        if self.cache.contains_key(&key) {
            self.cache.insert(key.clone(), value);
            self.mark_recent(&key);
            return None;
        }

        self.priority.push_front(key.clone());
        self.cache.insert(key, value);

        if self.cache.len() > SIZE {
            let evicted_key = self
                .priority
                .pop_back()
                .expect("lru priority queue should contain an eviction candidate");
            let evicted_value = self
                .cache
                .remove(&evicted_key)
                .expect("lru cache should contain the evicted key");
            Some((evicted_key, evicted_value))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LruCache;

    #[test]
    fn get_marks_entry_as_recently_used() {
        let mut cache = LruCache::<i32, &'static str, 2>::default();

        assert_eq!(cache.put(1, "one"), None);
        assert_eq!(cache.put(2, "two"), None);
        assert_eq!(cache.get(&1), Some(&"one"));

        assert_eq!(cache.put(3, "three"), Some((2, "two")));
        assert!(cache.contains_key(&1));
        assert!(!cache.contains_key(&2));
        assert!(cache.contains_key(&3));
    }

    #[test]
    fn missing_get_does_not_change_priority() {
        let mut cache = LruCache::<i32, &'static str, 2>::default();

        cache.put(1, "one");
        cache.put(2, "two");
        assert_eq!(cache.get(&99), None);

        assert_eq!(cache.put(3, "three"), Some((1, "one")));
    }

    #[test]
    fn take_removes_and_returns_owned_value() {
        let mut cache = LruCache::<i32, String, 2>::default();

        cache.put(1, "one".to_owned());
        assert_eq!(cache.take(&1), Some("one".to_owned()));
        assert_eq!(cache.take(&1), None);
        assert!(cache.is_empty());
    }

    #[test]
    fn replacing_existing_value_does_not_evict() {
        let mut cache = LruCache::<i32, &'static str, 2>::default();

        cache.put(1, "one");
        cache.put(2, "two");
        assert_eq!(cache.put(1, "updated"), None);

        assert_eq!(cache.put(3, "three"), Some((2, "two")));
        assert_eq!(cache.get(&1), Some(&"updated"));
    }

    #[test]
    fn zero_sized_cache_does_not_store_values() {
        let mut cache = LruCache::<i32, &'static str, 0>::default();

        assert_eq!(cache.put(1, "one"), Some((1, "one")));
        assert!(cache.is_empty());
        assert_eq!(cache.get(&1), None);
    }
}
