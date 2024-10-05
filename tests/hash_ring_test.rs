#[cfg(test)]
mod test {
    use std::fmt::Display;
    use pandas_pouch::hash_ring::{HashRing, RingNodeInfo};
    use std::hash::BuildHasherDefault;
    use std::hash::Hasher;

    // Defines a NodeInfo for a localhost address with a given port.
    fn node(port: u16) -> RingNodeInfo {
        RingNodeInfo {
            host: "localhost".to_string(),
            port,
        }
    }

    #[test]
    fn test_empty_ring() {
        let hash_ring: HashRing<RingNodeInfo> = HashRing::new(vec![], 10);
        assert_eq!(None, hash_ring.get_node("hello".to_string()));
    }

    #[test]
    fn test_default_nodes() {
        let mut nodes: Vec<RingNodeInfo> = Vec::new();
        nodes.push(node(15324));
        nodes.push(node(15325));
        nodes.push(node(15326));
        nodes.push(node(15327));
        nodes.push(node(15328));
        nodes.push(node(15329));

        let mut hash_ring: HashRing<RingNodeInfo> = HashRing::new(nodes, 10);

        assert_eq!(Some(&node(15324)), hash_ring.get_node("two".to_string()));
        assert_eq!(Some(&node(15325)), hash_ring.get_node("seven".to_string()));
        assert_eq!(Some(&node(15326)), hash_ring.get_node("hello".to_string()));
        assert_eq!(Some(&node(15327)), hash_ring.get_node("dude".to_string()));
        assert_eq!(Some(&node(15328)), hash_ring.get_node("fourteen".to_string()));
        assert_eq!(Some(&node(15329)), hash_ring.get_node("five".to_string()));

        hash_ring.remove_node(&node(15329));
        assert_eq!(Some(&node(15326)), hash_ring.get_node("hello".to_string()));

        hash_ring.add_node(&node(15329));
        assert_eq!(Some(&node(15326)), hash_ring.get_node("hello".to_string()));
    }

    #[derive(Clone)]
    struct CustomNodeInfo {
        pub host: &'static str,
        pub port: u16,
    }

    impl Display for CustomNodeInfo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", format!("{}:{}", self.host, self.port))
        }
    }

    #[test]
    fn test_custom_nodes() {
        let mut nodes: Vec<CustomNodeInfo> = Vec::new();
        nodes.push(CustomNodeInfo {
            host: "localhost",
            port: 15324,
        });
        nodes.push(CustomNodeInfo {
            host: "localhost",
            port: 15325,
        });
        nodes.push(CustomNodeInfo {
            host: "localhost",
            port: 15326,
        });
        nodes.push(CustomNodeInfo {
            host: "localhost",
            port: 15327,
        });
        nodes.push(CustomNodeInfo {
            host: "localhost",
            port: 15328,
        });
        nodes.push(CustomNodeInfo {
            host: "localhost",
            port: 15329,
        });

        let mut hash_ring: HashRing<CustomNodeInfo> = HashRing::new(nodes, 10);

        assert_eq!(
            Some("localhost:15326".to_string()),
            hash_ring
                .get_node("hello".to_string())
                .map(|x| x.to_string(),)
        );
        assert_eq!(
            Some("localhost:15327".to_string()),
            hash_ring
                .get_node("dude".to_string())
                .map(|x| x.to_string(),)
        );

        hash_ring.remove_node(&CustomNodeInfo {
            host: "localhost",
            port: 15329,
        });
        assert_eq!(
            Some("localhost:15326".to_string()),
            hash_ring
                .get_node("hello".to_string())
                .map(|x| x.to_string(),)
        );

        hash_ring.add_node(&CustomNodeInfo {
            host: "localhost",
            port: 15329,
        });
        assert_eq!(
            Some("localhost:15326".to_string()),
            hash_ring
                .get_node("hello".to_string())
                .map(|x| x.to_string(),)
        );
    }

    #[test]
    fn test_remove_actual_node() {
        let mut nodes: Vec<RingNodeInfo> = Vec::new();
        nodes.push(node(15324));
        nodes.push(node(15325));
        nodes.push(node(15326));
        nodes.push(node(15327));
        nodes.push(node(15328));
        nodes.push(node(15329));

        let mut hash_ring: HashRing<RingNodeInfo> = HashRing::new(nodes, 10);

        // This should be num nodes * num replicas
        assert_eq!(60, hash_ring.sorted_keys.len());
        assert_eq!(60, hash_ring.ring.len());

        hash_ring.remove_node(&node(15326));

        // This should be num nodes * num replicas
        assert_eq!(50, hash_ring.sorted_keys.len());
        assert_eq!(50, hash_ring.ring.len());
    }

    #[test]
    fn test_remove_non_existent_node() {
        let mut nodes: Vec<RingNodeInfo> = Vec::new();
        nodes.push(node(15324));
        nodes.push(node(15325));
        nodes.push(node(15326));
        nodes.push(node(15327));
        nodes.push(node(15328));
        nodes.push(node(15329));

        let mut hash_ring: HashRing<RingNodeInfo> = HashRing::new(nodes, 10);

        hash_ring.remove_node(&node(15330));

        // This should be num nodes * num replicas
        assert_eq!(60, hash_ring.sorted_keys.len());
        assert_eq!(60, hash_ring.ring.len());
    }

    #[test]
    fn test_custom_hasher() {
        #[derive(Default)]
        struct ConstantHasher;

        impl Hasher for ConstantHasher {
            fn finish(&self) -> u64 {
                1
            }

            fn write(&mut self, _bytes: &[u8]) {
                // Do nothing
            }
        }

        type ConstantBuildHasher = BuildHasherDefault<ConstantHasher>;

        let mut nodes: Vec<RingNodeInfo> = Vec::new();
        nodes.push(node(15324));
        nodes.push(node(15325));
        nodes.push(node(15326));
        nodes.push(node(15327));
        nodes.push(node(15328));
        nodes.push(node(15329));

        let hash_ring: HashRing<RingNodeInfo, ConstantBuildHasher> =
            HashRing::with_hasher(nodes, 10, ConstantBuildHasher::default());

        assert_eq!(Some(&node(15329)), hash_ring.get_node("hello".to_string()));
        assert_eq!(Some(&node(15329)), hash_ring.get_node("dude".to_string()));
        assert_eq!(Some(&node(15329)), hash_ring.get_node("two".to_string()));
    }
}