use pandas_pouch::lru::LRUCache;
use std::thread;
use std::time::Duration;

fn main() {
    let mut cache = LRUCache::new(2, Some(Duration::from_secs(2)));

    println!("LRUCache created with capacity 2 and expiry of 2 seconds");

    cache.put(1, "a");
    println!("1: {:?}", cache.get(&1));
    cache.put(2, "b");
    println!("2: {:?}", cache.get(&2));

    println!("Printing...");
    println!("{:?}", cache.print());

    println!("Sleeping for 5 secs...");
    thread::sleep(Duration::from_secs(5));
    println!("Woke up!");

    println!("1: {:?}", cache.get(&1));

    cache.put(1, "i");
    cache.put(3, "c");
    println!("1: {:?}", cache.get(&1));
    println!("2: {:?}", cache.get(&2));
    println!("3: {:?}", cache.get(&3));

    cache.put(4, "d");
    println!("2: {:?}", cache.get(&2));
    println!("3: {:?}", cache.get(&3));
    println!("4: {:?}", cache.get(&4));

    println!("Printing...");
    println!("{:?}", cache.print());
}
