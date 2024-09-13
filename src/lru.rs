// Least Recently Used Implementation for Caching

use std::fmt::Display;
use std::hash::Hash;
use std::time::{Duration, Instant};
use parking_lot::Mutex;
use dashmap::DashMap;
use std::sync::Arc;

pub type Link<K, V> = Option<Arc<Mutex<Node<K, V>>>>;

pub struct Node<K, V> {
    key: K,
    value: V,
    expires_at: Instant,
    prev: Link<K, V>,
    next: Link<K, V>,
}

pub struct LRUCache<K: Eq + Hash, V> {
    map: DashMap<K, Link<K, V>>,
    expires: Duration,
    head: Link<K, V>,
    tail: Link<K, V>,
    capacity: usize,
}

impl<K: Eq + Hash + Clone + Display, V: Clone + Display> LRUCache<K, V> {
    pub fn new(capacity: usize, expires: Option<Duration>) -> LRUCache<K, V> {
        let expires = expires.unwrap_or(Duration::from_secs(3600));
        LRUCache {
            map: DashMap::new(),
            expires,
            head: None,
            tail: None,
            capacity,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        let node_ref = self.map.get(key)?.as_ref()?.clone();
        let node = node_ref.lock();
        if node.expires_at < Instant::now() {
            drop(node);
            self.remove(key.clone());
            None
        } else {
            let value = node.value.clone();
            drop(node);
            self.move_to_head(node_ref);
            Some(value)
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        if let Some(node_ref) = self.map.get(&key).and_then(|r| r.value().clone()) {
            let mut node = node_ref.lock();
            node.value = value;
            node.expires_at = Instant::now() + self.expires;
            drop(node);
            self.move_to_head(node_ref);
        } else {
            let new_node = Arc::new(Mutex::new(Node {
                key: key.clone(),
                value,
                expires_at: Instant::now() + self.expires,
                prev: None,
                next: self.head.clone(),
            }));

            if let Some(head) = &self.head {
                let mut head = head.lock();
                head.prev = Some(Arc::clone(&new_node));
            } else {
                self.tail = Some(Arc::clone(&new_node));
            }
            self.head = Some(Arc::clone(&new_node));

            if self.map.len() >= self.capacity {
                if let Some(tail) = self.tail.clone() {
                    let tail = tail.lock();
                    let prev = tail.prev.clone();
                    let key_to_remove = tail.key.clone();
                    drop(tail);
                    self.map.remove(&key_to_remove);
                    self.tail = prev;
                    if let Some(new_tail) = &self.tail {
                        new_tail.lock().next = None;
                    }
                }
            }

            self.map.insert(key, Some(new_node));
        }
    }

    pub fn print(&mut self) -> Vec<(K, V)> {
        let mut current = self.head.clone();
        let mut get_all = Vec::new();
        while let Some(node) = current {
            let node_lock = node.lock();
            let key = node_lock.key.clone();
            let value = node_lock.value.clone();
            let expires_at = node_lock.expires_at;
            
            current = node_lock.next.clone();
            drop(node_lock);

            if expires_at < Instant::now() {
                self.remove(key);
            } else {
                get_all.push((key, value));
            }
        }
        get_all
    }

    fn detach_node(&mut self, node_ref: Arc<Mutex<Node<K, V>>>) {
        let mut node = node_ref.lock();
        let prev = node.prev.clone();
        let next = node.next.clone();

        if let Some(prev_node_ref) = &prev {
            prev_node_ref.lock().next = next.clone();
        } else {
            // node is head of LRUCache DLL, update the head
            self.head = next.clone();
        }

        if let Some(next_node_ref) = &next {
            next_node_ref.lock().prev = prev;
        } else {
            // node is in tail, update tail
            self.tail = prev;
        }

        node.prev = None;
        node.next = None;
    }

    fn remove(&mut self, key: K) -> Option<(K, V)> {
        if let Some((_, node_link)) = self.map.remove(&key) {
            if let Some(node_ref) = node_link {
                // unlink/detaching node from DLL
                self.detach_node(node_ref.clone());
                let node = node_ref.lock();
                return Some((node.key.clone(), node.value.clone()));
            }
        }
        None
    }

    fn move_to_head(&mut self, node_ref: Arc<Mutex<Node<K, V>>>) {
        // unlinking/detaching node from the DLL
        self.detach_node(node_ref.clone());

        // inserting at head
        let mut node = node_ref.lock();
        node.next = self.head.clone();
        node.prev = None;
        drop(node);

        if let Some(head_ref) = &self.head {
            head_ref.lock().prev = Some(node_ref.clone());
        } else {
            // DLL is empty, both head and tail to the node
            self.tail = Some(node_ref.clone());
        }

        self.head = Some(node_ref);
    }
}

impl<K: Eq + Hash, V> Drop for LRUCache<K, V> {
    fn drop(&mut self) {
        // Clear the map to break potential circular references
        self.map.clear();
        // Set head and tail to None to break the linked list
        self.head = None;
        self.tail = None;
    }
}
