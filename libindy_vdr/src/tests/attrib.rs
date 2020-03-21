use crate::common::did::DidValue;
use crate::ledger::constants;
use crate::tests::utils::helpers;
use crate::tests::utils::pool::*;

lazy_static! {
    static ref ATTRIB_RAW_DATA: serde_json::Value = json!({"endpoint":{"ha":"127.0.0.1:5555"}});
}

pub const ATTRIB_RAW_DATA_FIELD: &'static str = r#"endpoint"#;
pub const ATTRIB_HASH_DATA: &'static str =
    r#"83d907821df1c87db829e96569a11f6fc2e7880acba5e43d07ab786959e13bd3"#;
pub const ATTRIB_ENC_DATA: &'static str = r#"aa3f41f619aa7e5e6b6d0de555e05331787f9bf9aa672b94b57ab65b9b66c3ea960b18a98e3834b1fc6cebf49f463b81fd6e3181"#;

#[cfg(test)]
mod builder {
    use super::*;
    use crate::tests::utils::constants::{MY1_DID, MY1_DID_FQ, TRUSTEE_DID, TRUSTEE_DID_FQ};

    mod attrib {
        use super::*;

        #[test]
        fn test_pool_build_attrib_requests_works_for_raw_value() {
            let pool = TestPool::new();

            let attrib_request = pool
                .request_builder()
                .build_attrib_request(
                    &DidValue(String::from(TRUSTEE_DID)),
                    &DidValue(String::from(MY1_DID)),
                    None,
                    Some(&ATTRIB_RAW_DATA),
                    None,
                )
                .unwrap();

            let expected_result = json!({
                "type": constants::ATTRIB,
                "dest": MY1_DID,
                "raw": ATTRIB_RAW_DATA.to_string()
            });

            helpers::check_request_operation(&attrib_request, expected_result);
        }

        #[test]
        fn test_pool_build_attrib_requests_works_for_hash_value() {
            let pool = TestPool::new();

            let attrib_request = pool
                .request_builder()
                .build_attrib_request(
                    &DidValue(String::from(TRUSTEE_DID)),
                    &DidValue(String::from(MY1_DID)),
                    Some(ATTRIB_HASH_DATA.to_string()),
                    None,
                    None,
                )
                .unwrap();

            let expected_result = json!({
                "type": constants::ATTRIB,
                "dest": MY1_DID,
                "hash": ATTRIB_HASH_DATA
            });

            helpers::check_request_operation(&attrib_request, expected_result);
        }

        #[test]
        fn test_pool_build_attrib_requests_works_for_enc_value() {
            let pool = TestPool::new();

            let attrib_request = pool
                .request_builder()
                .build_attrib_request(
                    &DidValue(String::from(TRUSTEE_DID)),
                    &DidValue(String::from(MY1_DID)),
                    None,
                    None,
                    Some(ATTRIB_ENC_DATA.to_string()),
                )
                .unwrap();

            let expected_result = json!({
                "type": constants::ATTRIB,
                "dest": MY1_DID,
                "enc": ATTRIB_ENC_DATA
            });

            helpers::check_request_operation(&attrib_request, expected_result);
        }

        #[test]
        fn test_pool_build_attrib_requests_works_for_fully_qualified_dids() {
            let pool = TestPool::new();

            let attrib_request = pool
                .request_builder()
                .build_attrib_request(
                    &DidValue(String::from(TRUSTEE_DID_FQ)),
                    &DidValue(String::from(MY1_DID_FQ)),
                    None,
                    Some(&ATTRIB_RAW_DATA),
                    None,
                )
                .unwrap();

            let expected_result = json!({
                "type": constants::ATTRIB,
                "dest": MY1_DID,
                "raw": ATTRIB_RAW_DATA.to_string()
            });

            helpers::check_request_operation(&attrib_request, expected_result);
        }
    }

    mod get_attrib {
        use super::*;

        #[test]
        fn test_pool_build_get_attrib_requests_works_for_raw_value() {
            let pool = TestPool::new();

            let attrib_request = pool
                .request_builder()
                .build_get_attrib_request(
                    None,
                    &DidValue(String::from(MY1_DID)),
                    Some(ATTRIB_RAW_DATA_FIELD.to_string()),
                    None,
                    None,
                )
                .unwrap();

            let expected_result = json!({
                "type": constants::GET_ATTR,
                "dest": MY1_DID,
                "raw": ATTRIB_RAW_DATA_FIELD
            });

            helpers::check_request_operation(&attrib_request, expected_result);
        }

        #[test]
        fn test_pool_build_get_attrib_requests_works_for_hash_value() {
            let pool = TestPool::new();

            let attrib_request = pool
                .request_builder()
                .build_get_attrib_request(
                    None,
                    &DidValue(String::from(MY1_DID)),
                    None,
                    Some(ATTRIB_HASH_DATA.to_string()),
                    None,
                )
                .unwrap();

            let expected_result = json!({
                "type": constants::GET_ATTR,
                "dest": MY1_DID,
                "hash": ATTRIB_HASH_DATA
            });

            helpers::check_request_operation(&attrib_request, expected_result);
        }

        #[test]
        fn test_pool_build_get_attrib_requests_works_for_enc_value() {
            let pool = TestPool::new();

            let attrib_request = pool
                .request_builder()
                .build_get_attrib_request(
                    None,
                    &DidValue(String::from(MY1_DID)),
                    None,
                    None,
                    Some(ATTRIB_ENC_DATA.to_string()),
                )
                .unwrap();

            let expected_result = json!({
                "type": constants::GET_ATTR,
                "dest": MY1_DID,
                "enc": ATTRIB_ENC_DATA
            });

            helpers::check_request_operation(&attrib_request, expected_result);
        }

        #[test]
        fn test_pool_build_get_attrib_requests_works_for_fully_qualified_dids() {
            let pool = TestPool::new();

            let attrib_request = pool
                .request_builder()
                .build_get_attrib_request(
                    None,
                    &DidValue(String::from(MY1_DID_FQ)),
                    None,
                    None,
                    Some(ATTRIB_ENC_DATA.to_string()),
                )
                .unwrap();

            let expected_result = json!({
                "type": constants::GET_ATTR,
                "dest": MY1_DID,
                "enc": ATTRIB_ENC_DATA
            });

            helpers::check_request_operation(&attrib_request, expected_result);
        }
    }
}

#[cfg(test)]
mod send_attrib {
    use super::*;
    use crate::tests::utils::crypto::Identity;

    #[test]
    fn test_pool_send_attrib_request_for_raw_value() {
        let pool = TestPool::new();
        let identity = helpers::new_ledger_identity(&pool, None);

        // Send Attrib
        let mut attrib_request = pool
            .request_builder()
            .build_attrib_request(
                &identity.did,
                &identity.did,
                None,
                Some(&ATTRIB_RAW_DATA),
                None,
            )
            .unwrap();

        identity.sign_request(&mut attrib_request);

        let attrib_response = pool.send_request(&attrib_request).unwrap();

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
            ATTRIB_RAW_DATA.to_string(),
            helpers::get_response_data(&response).unwrap()
        );
    }

    #[test]
    fn test_pool_send_attrib_request_for_hash_value() {
        let pool = TestPool::new();
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

        identity.sign_request(&mut attrib_request);

        let attrib_response = pool.send_request(&attrib_request).unwrap();

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

    #[test]
    fn test_pool_send_attrib_request_for_enc_value() {
        let pool = TestPool::new();
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

        identity.sign_request(&mut attrib_request);

        let attrib_response = pool.send_request(&attrib_request).unwrap();

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

    #[test]
    fn test_pool_send_attrib_request_without_signature() {
        let pool = TestPool::new();
        let trustee = Identity::trustee();

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

    #[test]
    fn test_pool_send_attrib_request_for_unknown_target_identity() {
        let pool = TestPool::new();
        let trustee = Identity::trustee();
        let new_identity = Identity::new(None);

        // Send Attrib
        let mut attrib_request = pool
            .request_builder()
            .build_attrib_request(
                &trustee.did,
                &new_identity.did,
                Some(ATTRIB_HASH_DATA.to_string()),
                None,
                None,
            )
            .unwrap();

        trustee.sign_request(&mut attrib_request);

        let err = pool.send_request(&attrib_request).unwrap_err();
        helpers::check_response_type(&err, "REJECT");
    }

    #[test]
    fn test_pool_send_attrib_request_for_unknown_sender() {
        let pool = TestPool::new();
        let identity = Identity::new(None);

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

        identity.sign_request(&mut attrib_request);

        let err = pool.send_request(&attrib_request).unwrap_err();
        helpers::check_response_type(&err, "REQNACK");
    }

    #[test]
    fn test_pool_send_attrib_request_for_touching_other_identity() {
        let pool = TestPool::new();
        let trustee = Identity::trustee();
        let identity = helpers::new_ledger_identity(&pool, None);

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

        trustee.sign_request(&mut attrib_request);

        let err = pool.send_request(&attrib_request).unwrap_err();
        helpers::check_response_type(&err, "REJECT");
    }
}
