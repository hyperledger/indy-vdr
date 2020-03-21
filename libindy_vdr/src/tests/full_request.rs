use crate::tests::utils::crypto::*;
use crate::tests::utils::pool::*;

#[test]
fn test_pool_send_full_request_works() {
    let pool = TestPool::new();
    let trustee = Identity::trustee();

    let mut pool_restart_request = pool
        .request_builder()
        .build_get_validator_info_request(&trustee.did)
        .unwrap();

    trustee.sign_request(&mut pool_restart_request);

    let replies = pool
        .send_full_request(&pool_restart_request, None, None)
        .unwrap();

    assert_eq!(replies.len(), pool.transactions().len());
    assert!(replies.contains_key("Node1"));
    assert!(replies.contains_key("Node2"));
    assert!(replies.contains_key("Node3"));
    assert!(replies.contains_key("Node4"));
}

#[test]
fn test_pool_send_full_request_works_for_list_nodes() {
    let pool = TestPool::new();
    let trustee = Identity::trustee();

    let mut pool_restart_request = pool
        .request_builder()
        .build_get_validator_info_request(&trustee.did)
        .unwrap();

    trustee.sign_request(&mut pool_restart_request);

    let replies = pool
        .send_full_request(
            &pool_restart_request,
            Some(vec![String::from("Node1"), String::from("Node2")]),
            None,
        )
        .unwrap();
    assert_eq!(replies.len(), 2);
    assert!(replies.contains_key("Node1"));
    assert!(replies.contains_key("Node2"));
}

#[test]
fn test_pool_send_full_request_works_for_timeout() {
    let pool = TestPool::new();
    let trustee = Identity::trustee();

    let mut pool_restart_request = pool
        .request_builder()
        .build_get_validator_info_request(&trustee.did)
        .unwrap();

    trustee.sign_request(&mut pool_restart_request);

    let replies = pool
        .send_full_request(&pool_restart_request, None, Some(100))
        .unwrap();
    assert_eq!(replies.len(), pool.transactions().len());
    assert!(replies.contains_key("Node1"));
    assert!(replies.contains_key("Node2"));
    assert!(replies.contains_key("Node3"));
    assert!(replies.contains_key("Node4"));
}

#[test]
fn test_pool_send_full_request_works_for_unknown_node() {
    let pool = TestPool::new();
    let trustee = Identity::trustee();

    let mut pool_restart_request = pool
        .request_builder()
        .build_get_validator_info_request(&trustee.did)
        .unwrap();

    trustee.sign_request(&mut pool_restart_request);

    let _err = pool
        .send_full_request(
            &pool_restart_request,
            Some(vec![String::from("UNKNOWN")]),
            None,
        )
        .unwrap_err();
    println!("_err {:?}", _err);
}
