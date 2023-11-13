#[macro_use]
mod utils;

inject_dependencies!();

#[test]
fn empty() {
    // Empty test to run module
}

#[cfg(test)]
#[cfg(feature = "local_nodes_pool")]
mod send_resolver {
    use futures_executor::block_on;
    use indy_vdr::resolver::PoolResolver as Resolver;

    use crate::utils::crypto::Identity;
    use crate::utils::fixtures::*;
    use crate::utils::helpers;
    use crate::utils::pool::TestPool;

    #[rstest]
    fn test_pool_resolve_did(
        pool: TestPool,
        trustee: Identity,
        identity: Identity,
        diddoc_content: serde_json::Value,
    ) {
        // Send NYM
        let mut nym_request = pool
            .request_builder()
            .build_nym_request(
                &trustee.did,
                &identity.did,
                Some(identity.verkey.to_string()),
                None,
                None,
                Some(&diddoc_content),
                None,
            )
            .unwrap();

        let nym_response =
            helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap();

        // Get NYM to make sure it was written before it gets resolved
        let get_nym_request = pool
            .request_builder()
            .build_get_nym_request(None, &identity.did, None, None)
            .unwrap();

        let _response = pool
            .send_request_with_retries(&get_nym_request, &nym_response)
            .unwrap();

        // Resolve DID
        let resolver = Resolver::new(pool.pool);
        let qualified_did = format!("did:indy:test:{}", &identity.did);
        let result = block_on(resolver.resolve(&qualified_did)).unwrap();

        let v: serde_json::Value = serde_json::from_str(result.as_str()).unwrap();

        let diddoc = &v["didDocument"];
        let metadata = &v["didDocumentMetadata"];

        assert_eq!("NYM", metadata["objectType"]);
        assert_ne!(&serde_json::Value::Null, diddoc)
    }

    #[rstest]
    fn test_pool_resolve_did_with_version_id(
        pool: TestPool,
        trustee: Identity,
        identity: Identity,
    ) {
        // Send NYM
        let mut nym_request = pool
            .request_builder()
            .build_nym_request(
                &trustee.did,
                &identity.did,
                Some(identity.verkey.to_string()),
                None,
                None,
                None,
                None,
            )
            .unwrap();

        let nym_response =
            helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap();
        println!("nym response: {nym_response}");

        // Get NYM to make sure it was written before it gets resolved
        let get_nym_request = pool
            .request_builder()
            .build_get_nym_request(None, &identity.did, None, None)
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_nym_request, &nym_response)
            .unwrap();
        println!("get nym response: {nym_response}");
        let seq_no = TestPool::extract_seq_no_from_reply(response.as_str()).unwrap();

        // Resolve DID
        let resolver = Resolver::new(pool.pool);
        let qualified_did = format!("did:indy:test:{}", &identity.did);
        let did_url = format!("{}?versionId={}", qualified_did, seq_no);
        let result = block_on(resolver.resolve(&did_url)).unwrap();

        let v: serde_json::Value = serde_json::from_str(result.as_str()).unwrap();

        let diddoc = &v["didDocument"];
        let metadata = &v["didDocumentMetadata"];

        assert_eq!("NYM", metadata["objectType"]);
        assert_ne!(&serde_json::Value::Null, diddoc);
    }
}
