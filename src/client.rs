use crate::lru::LRUCache;
use std::fmt::Display;
use std::hash::Hash;
use std::time::Duration;

pub struct Client<K: Eq + Hash, V> {
    cache: LRUCache<K, V>,
}

impl<K: Eq + std::hash::Hash + Clone + Display, V: Clone + Display> Client<K, V> {
    pub fn new(
        _host: &str,
        _port: u16,
        capacity: usize,
        expires: Option<Duration>,
    ) -> Client<K, V> {
        // todo! : to use host and port to connect to a remote server
        let cache = LRUCache::new(capacity, expires);
        Client { cache }
    }

    pub fn get(&mut self, key: K) -> Option<V> {
        self.cache.get(&key)
    }

    pub fn put(&mut self, key: K, value: V) {
        self.cache.put(key, value);
    }
}
