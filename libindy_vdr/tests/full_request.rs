#[macro_use]
mod utils;

inject_dependencies!();

#[test]
fn empty() {
    // Empty test to run module
}

#[cfg(test)]
#[cfg(feature = "local_nodes_pool")]
mod send_full_request {
    use crate::utils::crypto::Identity;
    use crate::utils::fixtures::*;
    use crate::utils::helpers;
    use crate::utils::pool::TestPool;

    #[rstest]
    fn test_pool_send_full_request_works(pool: TestPool, trustee: Identity) {
        let replies = helpers::sign_and_send_full_request(&pool, &trustee, None, None).unwrap();

        assert_eq!(replies.len(), pool.transactions().len());
        assert!(replies.contains_key("Node1"));
        assert!(replies.contains_key("Node2"));
        assert!(replies.contains_key("Node3"));
        assert!(replies.contains_key("Node4"));
    }

    #[rstest]
    fn test_pool_send_full_request_works_for_list_nodes(pool: TestPool, trustee: Identity) {
        let replies = helpers::sign_and_send_full_request(
            &pool,
            &trustee,
            Some(vec![String::from("Node1"), String::from("Node2")]),
            None,
        )
        .unwrap();

        assert_eq!(replies.len(), 2);
        assert!(replies.contains_key("Node1"));
        assert!(replies.contains_key("Node2"));
    }

    #[rstest]
    fn test_pool_send_full_request_works_for_timeout(pool: TestPool, trustee: Identity) {
        let replies =
            helpers::sign_and_send_full_request(&pool, &trustee, None, Some(100)).unwrap();

        assert_eq!(replies.len(), pool.transactions().len());
        assert!(replies.contains_key("Node1"));
        assert!(replies.contains_key("Node2"));
        assert!(replies.contains_key("Node3"));
        assert!(replies.contains_key("Node4"));
    }

    #[rstest]
    fn test_pool_send_full_request_works_for_unknown_node(pool: TestPool, trustee: Identity) {
        let _err = helpers::sign_and_send_full_request(
            &pool,
            &trustee,
            Some(vec![String::from("UNKNOWN")]),
            None,
        )
        .unwrap_err();
    }
}
