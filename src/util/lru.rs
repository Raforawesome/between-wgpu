use std::hash::Hash;

use crate::HashMap;

/// Fixed-size least-recently-used cache with O(1) operations.
///
/// The front of the linked list is the most recently used key and the back is the
/// least recently used key. Values are owned by the cache so callers can move
/// them back out with [`LruCache::take`] when needed.
pub struct LruCache<K, V, const SIZE: usize>
where
    K: Eq + Hash + Clone,
{
    map: HashMap<K, usize>,
    nodes: Vec<Option<Node<K, V>>>,
    head: Option<usize>,
    tail: Option<usize>,
    free: Vec<usize>,
}

struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
}

impl<K, V, const SIZE: usize> Default for LruCache<K, V, SIZE>
where
    K: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self {
            map: HashMap::default(),
            nodes: Vec::with_capacity(SIZE),
            head: None,
            tail: None,
            free: Vec::new(),
        }
    }
}

impl<K, V, const SIZE: usize> LruCache<K, V, SIZE>
where
    K: Eq + Hash + Clone,
{
    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    fn detach(&mut self, idx: usize) {
        let node = self.nodes[idx].as_ref().expect("node should exist");
        let prev = node.prev;
        let next = node.next;

        if let Some(p) = prev {
            self.nodes[p].as_mut().expect("prev node should exist").next = next;
        } else {
            self.head = next;
        }

        if let Some(n) = next {
            self.nodes[n].as_mut().expect("next node should exist").prev = prev;
        } else {
            self.tail = prev;
        }
    }

    fn attach_front(&mut self, idx: usize) {
        let node = self.nodes[idx].as_mut().expect("node should exist");
        node.prev = None;
        node.next = self.head;

        if let Some(h) = self.head {
            self.nodes[h].as_mut().expect("head node should exist").prev = Some(idx);
        } else {
            self.tail = Some(idx);
        }
        self.head = Some(idx);
    }

    /// Returns a reference to a cached value and marks it as recently used.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let idx = *self.map.get(key)?;
        self.detach(idx);
        self.attach_front(idx);
        Some(&self.nodes[idx].as_ref().unwrap().value)
    }

    /// Returns a mutable reference to a cached value and marks it as recently used.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let idx = *self.map.get(key)?;
        self.detach(idx);
        self.attach_front(idx);
        Some(&mut self.nodes[idx].as_mut().unwrap().value)
    }

    /// Returns a reference to a cached value without modifying its priority.
    pub fn peek(&self, key: &K) -> Option<&V> {
        let idx = *self.map.get(key)?;
        Some(&self.nodes[idx].as_ref().unwrap().value)
    }

    /// Returns a mutable reference to a cached value without modifying its priority.
    pub fn peek_mut(&mut self, key: &K) -> Option<&mut V> {
        let idx = *self.map.get(key)?;
        Some(&mut self.nodes[idx].as_mut().unwrap().value)
    }

    /// Removes a value from the cache and returns ownership of it.
    pub fn take(&mut self, key: &K) -> Option<V> {
        let idx = self.map.remove(key)?;
        self.detach(idx);

        let node = self.nodes[idx].take().expect("node should exist");
        self.free.push(idx);
        Some(node.value)
    }

    /// Removes and returns the least recently used entry.
    pub fn pop_lru(&mut self) -> Option<(K, V)> {
        let key = self.nodes[self.tail?].as_ref()?.key.clone();
        let value = self.take(&key)?;
        Some((key, value))
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

        if let Some(&idx) = self.map.get(&key) {
            let node = self.nodes[idx].as_mut().expect("node should exist");
            node.value = value;
            self.detach(idx);
            self.attach_front(idx);
            return None;
        }

        let idx = if let Some(free_idx) = self.free.pop() {
            self.nodes[free_idx] = Some(Node {
                key: key.clone(),
                value,
                prev: None,
                next: None,
            });
            free_idx
        } else if self.nodes.len() < SIZE {
            let new_idx = self.nodes.len();
            self.nodes.push(Some(Node {
                key: key.clone(),
                value,
                prev: None,
                next: None,
            }));
            new_idx
        } else {
            // Evict tail (least recently used)
            let evicted_idx = self.tail.expect("cache is full, tail must exist");
            self.detach(evicted_idx);

            let evicted_node = self.nodes[evicted_idx]
                .take()
                .expect("evicted node must exist");
            self.map.remove(&evicted_node.key);

            self.nodes[evicted_idx] = Some(Node {
                key: key.clone(),
                value,
                prev: None,
                next: None,
            });
            self.attach_front(evicted_idx);
            self.map.insert(key, evicted_idx);

            return Some((evicted_node.key, evicted_node.value));
        };

        self.attach_front(idx);
        self.map.insert(key, idx);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::LruCache;

    #[test]
    fn pop_lru_preserves_order_and_reuses_slots() {
        let mut cache = LruCache::<i32, i32, 2>::default();
        assert_eq!(cache.pop_lru(), None);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.get(&1);
        assert_eq!(cache.pop_lru(), Some((2, 20)));
        cache.put(3, 30);
        assert_eq!(cache.pop_lru(), Some((1, 10)));
        assert_eq!(cache.pop_lru(), Some((3, 30)));
        assert!(cache.is_empty());
    }

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
