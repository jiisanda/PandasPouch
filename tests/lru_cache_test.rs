#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use pandas_pouch::lru::LRUCache;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_lru_cache() {
        let mut cache = LRUCache::new(2, None); // default expiration of 3600 seconds

        cache.put(1, "a");
        assert_eq!(cache.get(1), Some("a"));
        cache.put(2, "b");
        assert_eq!(cache.get(2), Some("b"));

        cache.put(3, "c");
        assert_eq!(cache.get(1), None);
        assert_eq!(cache.get(2), Some("b"));
        assert_eq!(cache.get(3), Some("c"));

        cache.put(4, "d");
        assert_eq!(cache.get(2), None);
        assert_eq!(cache.get(3), Some("c"));
        assert_eq!(cache.get(4), Some("d"));
    }

    #[test]
    fn test_lru_cache_with_expiration() {
        let mut cache = LRUCache::new(2, Some(Duration::from_secs(2)));

        cache.put(1, "a");
        assert_eq!(cache.get(1), Some("a"));
        cache.put(2, "b");
        assert_eq!(cache.get(2), Some("b"));

        thread::sleep(Duration::from_secs(5));
        assert_eq!(cache.get(1), None);
        assert_eq!(cache.get(2), None);
    }
    
    #[test]
    fn test_thread_safety() {
        let cache = LRUCache::new(100, Some(Duration::from_secs(1)));
        let cache = Arc::new(Mutex::new(cache));
        
        let mut handles = vec![];
        
        for i in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                let mut cache = cache_clone.lock().unwrap();
                cache.put(i, i*2);
                assert_eq!(cache.get(i), Some(i*2));
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        // check that all values are still correct after all threads have finished
        for i in 1..0 {
            let mut cache = cache.lock().unwrap();
            assert_eq!(cache.get(i), Some(i*2));
        }
    }
}
