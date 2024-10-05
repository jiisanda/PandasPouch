// consistent hashing implementation

use std::collections::{BinaryHeap, HashMap};
use std::fmt;
use std::hash::{BuildHasher, BuildHasherDefault, Hasher};
use twox_hash::XxHash64;

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct RingNodeInfo {
    pub host: String,
    pub port: u16,
}

impl fmt::Display for RingNodeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

type XxHash64Hasher = BuildHasherDefault<XxHash64>;

// HashRing
pub struct HashRing<T, S = XxHash64Hasher> {
    replicas: isize,                // replicas -> number of virtual nodes each node making a better distribution
    pub ring: HashMap<u64, T>,
    pub sorted_keys: Vec<u64>,
    hash_builder: S,
}

impl<T: ToString + Clone + fmt::Display> HashRing<T, XxHash64Hasher> {
    pub fn new(nodes: Vec<T>, replicas: isize) -> HashRing<T, XxHash64Hasher> {
        HashRing::with_hasher(nodes, replicas, XxHash64Hasher::default())
    }
}

impl<T, S> HashRing<T, S>
where
    T: ToString + Clone + fmt::Display,
    S: BuildHasher,
{
    pub fn with_hasher(nodes: Vec<T>, replicas: isize, hash_builder: S) -> HashRing<T, S> {
        let mut new_hash_ring: HashRing<T, S> = HashRing {
            replicas,
            ring: HashMap::new(),
            sorted_keys: Vec::new(),
            hash_builder,
        };

        for n in &nodes {
            new_hash_ring.add_node(n);
        }
        new_hash_ring
    }

    // add node
    pub fn add_node(&mut self, node: &T) {
        for i in 0..self.replicas {
            let key = self.gen_key(format!("{}:{}", node, i));
            self.ring.insert(key, (*node).clone());
            self.sorted_keys.push(key);
        }

        self.sorted_keys = BinaryHeap::from(self.sorted_keys.clone()).into_sorted_vec();
    }

    // delete node
    pub fn remove_node(&mut self, node: &T) {
        for i in 0..self.replicas {
            let key = self.gen_key(format!("{}:{}", node, i));
            if !self.ring.contains_key(&key) {
                return;
            }
            self.ring.remove(&key);
            let mut index = 0;
            for j in 0..self.sorted_keys.len() {
                if self.sorted_keys[j] == key {
                    index = j;
                    break;
                }
            }
            self.sorted_keys.remove(index);
        }
    }

    // gets the node a key belong to
    pub fn get_node(&self, key: String) -> Option<&T> {
        if self.sorted_keys.is_empty() {
            return None;
        }

        let generated_key  = self.gen_key(key);
        let nodes = self.sorted_keys.clone();

        for node in &nodes {
            if generated_key <= *node {
                return Some(self.ring.get(node).unwrap());
            }
        }

        let node = &nodes[0];

        Some(self.ring.get(node).unwrap())
    }

    // generates a key from a string value
    fn gen_key(&self, key: String) -> u64 {
        let mut hasher = self.hash_builder.build_hasher();
        hasher.write(key.as_bytes());
        hasher.finish()
    }
}
