#[macro_use]
mod utils;

inject_dependencies!();

use indy_vdr::ledger::constants;
use indy_vdr::utils::did::DidValue;

use crate::utils::fixtures::*;
use crate::utils::helpers;

const ATTRIB_RAW_DATA_FIELD: &str = r#"endpoint"#;
const ATTRIB_HASH_DATA: &str =
    r#"83d907821df1c87db829e96569a11f6fc2e7880acba5e43d07ab786959e13bd3"#;
const ATTRIB_ENC_DATA: &str = r#"aa3f41f619aa7e5e6b6d0de555e05331787f9bf9aa672b94b57ab65b9b66c3ea960b18a98e3834b1fc6cebf49f463b81fd6e3181"#;

fn attrib_raw_data() -> serde_json::Value {
    json!({ "endpoint": { "ha": "127.0.0.1:5555" } })
}

#[test]
fn empty() {
    // Empty test to run module
}

#[cfg(test)]
mod builder {
    use super::*;
    use indy_vdr::ledger::RequestBuilder;

    mod attrib {
        use super::*;

        #[rstest]
        fn test_pool_build_attrib_requests_works_for_raw_value(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
            my_did: DidValue,
        ) {
            let attrib_request = request_builder
                .build_attrib_request(&trustee_did, &my_did, None, Some(&attrib_raw_data()), None)
                .unwrap();

            let expected_result = json!({
                "type": constants::ATTRIB,
                "dest": my_did,
                "raw": attrib_raw_data().to_string()
            });

            helpers::check_request_operation(&attrib_request, expected_result);
        }

        #[rstest]
        fn test_pool_build_attrib_requests_works_for_hash_value(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
            my_did: DidValue,
        ) {
            let attrib_request = request_builder
                .build_attrib_request(
                    &trustee_did,
                    &my_did,
                    Some(ATTRIB_HASH_DATA.to_string()),
                    None,
                    None,
                )
                .unwrap();

            let expected_result = json!({
                "type": constants::ATTRIB,
                "dest": my_did,
                "hash": ATTRIB_HASH_DATA
            });

            helpers::check_request_operation(&attrib_request, expected_result);
        }

        #[rstest]
        fn test_pool_build_attrib_requests_works_for_enc_value(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
            my_did: DidValue,
        ) {
            let attrib_request = request_builder
                .build_attrib_request(
                    &trustee_did,
                    &my_did,
                    None,
                    None,
                    Some(ATTRIB_ENC_DATA.to_string()),
                )
                .unwrap();

            let expected_result = json!({
                "type": constants::ATTRIB,
                "dest": my_did,
                "enc": ATTRIB_ENC_DATA
            });

            helpers::check_request_operation(&attrib_request, expected_result);
        }

        #[rstest]
        fn test_pool_build_attrib_requests_works_for_fully_qualified_dids(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
            my_did: DidValue,
        ) {
            let attrib_request = request_builder
                .build_attrib_request(&trustee_did, &my_did, None, Some(&attrib_raw_data()), None)
                .unwrap();

            let expected_result = json!({
                "type": constants::ATTRIB,
                "dest": my_did,
                "raw": attrib_raw_data().to_string()
            });

            helpers::check_request_operation(&attrib_request, expected_result);
        }
    }

    mod get_attrib {
        use super::*;

        #[rstest]
        fn test_pool_build_get_attrib_requests_works_for_raw_value(
            request_builder: RequestBuilder,
            my_did: DidValue,
        ) {
            let attrib_request = request_builder
                .build_get_attrib_request(
                    None,
                    &my_did,
                    Some(ATTRIB_RAW_DATA_FIELD.to_string()),
                    None,
                    None,
                )
                .unwrap();

            let expected_result = json!({
                "type": constants::GET_ATTR,
                "dest": my_did,
                "raw": ATTRIB_RAW_DATA_FIELD
            });

            helpers::check_request_operation(&attrib_request, expected_result);
        }

        #[rstest]
        fn test_pool_build_get_attrib_requests_works_for_hash_value(
            request_builder: RequestBuilder,
            my_did: DidValue,
        ) {
            let attrib_request = request_builder
                .build_get_attrib_request(
                    None,
                    &my_did,
                    None,
                    Some(ATTRIB_HASH_DATA.to_string()),
                    None,
                )
                .unwrap();

            let expected_result = json!({
                "type": constants::GET_ATTR,
                "dest": my_did,
                "hash": ATTRIB_HASH_DATA
            });

            helpers::check_request_operation(&attrib_request, expected_result);
        }

        #[rstest]
        fn test_pool_build_get_attrib_requests_works_for_enc_value(
            request_builder: RequestBuilder,
            my_did: DidValue,
        ) {
            let attrib_request = request_builder
                .build_get_attrib_request(
                    None,
                    &my_did,
                    None,
                    None,
                    Some(ATTRIB_ENC_DATA.to_string()),
                )
                .unwrap();

            let expected_result = json!({
                "type": constants::GET_ATTR,
                "dest": my_did,
                "enc": ATTRIB_ENC_DATA
            });

            helpers::check_request_operation(&attrib_request, expected_result);
        }

        #[rstest]
        fn test_pool_build_get_attrib_requests_works_for_fully_qualified_dids(
            request_builder: RequestBuilder,
            fq_my_did: DidValue,
            my_did: DidValue,
        ) {
            let attrib_request = request_builder
                .build_get_attrib_request(
                    None,
                    &fq_my_did,
                    None,
                    None,
                    Some(ATTRIB_ENC_DATA.to_string()),
                )
                .unwrap();

            let expected_result = json!({
                "type": constants::GET_ATTR,
                "dest": my_did,
                "enc": ATTRIB_ENC_DATA
            });

            helpers::check_request_operation(&attrib_request, expected_result);
        }
    }
}

#[cfg(test)]
#[cfg(feature = "local_nodes_pool")]
mod send_attrib {
    use super::*;
    use crate::utils::crypto::Identity;
    use crate::utils::pool::TestPool;

    #[rstest]
    fn test_pool_send_attrib_request_for_raw_value(pool: TestPool) {
        let identity = helpers::new_ledger_identity(&pool, None);

        // Send Attrib
        let mut attrib_request = pool
            .request_builder()
            .build_attrib_request(
                &identity.did,
                &identity.did,
                None,
                Some(&attrib_raw_data()),
                None,
            )
            .unwrap();

        let attrib_response =
            helpers::sign_and_send_request(&identity, &pool, &mut attrib_request).unwrap();

        // Get Attrib
        let get_attrib_request = pool
            .request_builder()
            .build_get_attrib_request(
                None,
                &identity.did,
                Some(ATTRIB_RAW_DATA_FIELD.to_string()),
                None,
                None,
            )
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_attrib_request, &attrib_response)
            .unwrap();

        assert_eq!(
            attrib_raw_data().to_string(),
            helpers::get_response_data(&response).unwrap()
        );
    }

    #[rstest]
    fn test_pool_send_attrib_request_for_hash_value(pool: TestPool) {
        let identity = helpers::new_ledger_identity(&pool, None);

        // Send Attrib
        let mut attrib_request = pool
            .request_builder()
            .build_attrib_request(
                &identity.did,
                &identity.did,
                Some(ATTRIB_HASH_DATA.to_string()),
                None,
                None,
            )
            .unwrap();

        let attrib_response =
            helpers::sign_and_send_request(&identity, &pool, &mut attrib_request).unwrap();

        // Get Attrib
        let get_attrib_request = pool
            .request_builder()
            .build_get_attrib_request(
                None,
                &identity.did,
                None,
                Some(ATTRIB_HASH_DATA.to_string()),
                None,
            )
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_attrib_request, &attrib_response)
            .unwrap();

        assert_eq!(
            ATTRIB_HASH_DATA.to_string(),
            helpers::get_response_data(&response).unwrap()
        );
    }

    #[rstest]
    fn test_pool_send_attrib_request_for_enc_value(pool: TestPool) {
        let identity = helpers::new_ledger_identity(&pool, None);

        // Send Attrib
        let mut attrib_request = pool
            .request_builder()
            .build_attrib_request(
                &identity.did,
                &identity.did,
                None,
                None,
                Some(ATTRIB_ENC_DATA.to_string()),
            )
            .unwrap();

        let attrib_response =
            helpers::sign_and_send_request(&identity, &pool, &mut attrib_request).unwrap();

        // Get Attrib
        let get_attrib_request = pool
            .request_builder()
            .build_get_attrib_request(
                None,
                &identity.did,
                None,
                None,
                Some(ATTRIB_ENC_DATA.to_string()),
            )
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_attrib_request, &attrib_response)
            .unwrap();

        assert_eq!(
            ATTRIB_ENC_DATA.to_string(),
            helpers::get_response_data(&response).unwrap()
        );
    }

    #[rstest]
    fn test_pool_send_attrib_request_without_signature(pool: TestPool, trustee: Identity) {
        // Send Attrib
        let attrib_request = pool
            .request_builder()
            .build_attrib_request(
                &trustee.did,
                &trustee.did,
                Some(ATTRIB_HASH_DATA.to_string()),
                None,
                None,
            )
            .unwrap();

        let err = pool.send_request(&attrib_request).unwrap_err();
        helpers::check_response_type(&err, "REQNACK");
    }

    #[rstest]
    fn test_pool_send_attrib_request_for_unknown_target_identity(
        pool: TestPool,
        trustee: Identity,
        identity: Identity,
    ) {
        // Send Attrib
        let mut attrib_request = pool
            .request_builder()
            .build_attrib_request(
                &trustee.did,
                &identity.did,
                Some(ATTRIB_HASH_DATA.to_string()),
                None,
                None,
            )
            .unwrap();

        let err = helpers::sign_and_send_request(&trustee, &pool, &mut attrib_request).unwrap_err();
        helpers::check_response_type(&err, "REJECT");

        // Send Get Attrib
        let mut attrib_request = pool
            .request_builder()
            .build_get_attrib_request(
                None,
                &identity.did,
                Some(ATTRIB_RAW_DATA_FIELD.to_string()),
                None,
                None,
            )
            .unwrap();

        let response =
            helpers::sign_and_send_request(&trustee, &pool, &mut attrib_request).unwrap();
        helpers::get_response_data(&response).unwrap_err();
    }

    #[rstest]
    fn test_pool_send_attrib_request_for_unknown_sender(pool: TestPool, identity: Identity) {
        // Send Attrib
        let mut attrib_request = pool
            .request_builder()
            .build_attrib_request(
                &identity.did,
                &identity.did,
                Some(ATTRIB_HASH_DATA.to_string()),
                None,
                None,
            )
            .unwrap();

        let err =
            helpers::sign_and_send_request(&identity, &pool, &mut attrib_request).unwrap_err();
        helpers::check_response_type(&err, "REQNACK");
    }
}
