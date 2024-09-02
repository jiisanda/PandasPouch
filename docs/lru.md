# LRU - Least Recently Used Cache

> ! Will update more about this 

[//]: # (> TODO! Add details about LRU implementation...)

We will be using `DashMap`[`dashmap::DashMap`], instead of `HashMap` [`std::collection::HashMap`], there are few reasons,
for this update:
- HashMap is not thread safe, and requires setting up external synchronisation, for concurrent access. DashMap is
designed for concurrent use. It allows multiple threads to read and write simultaneously, without applying special locks.
- DashMap provides better performance in highly concurrent scenarios, it uses multiple internal shards, which intern
reduces contention between threads.
- HashMap is okay for single-threaded, but DashMap can be preferred in multithreaded environments, as we are expecting 
high concurrency.
