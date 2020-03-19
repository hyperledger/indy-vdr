use crate::tests::utils::pool::*;
use crate::tests::utils::crypto::Identity;
use crate::tests::utils::helpers;
use crate::common::did::DidValue;
use crate::ledger::constants;
use crate::ledger::requests::nym::role_to_code;

const ALIAS: &str = "alias";
const ROLE: &str = "TRUSTEE";

#[cfg(test)]
mod builder {
    use super::*;
    use crate::tests::utils::constants::{TRUSTEE_DID, MY1_DID, MY1_VERKEY, TRUSTEE_DID_FQ, MY1_DID_FQ};

    mod nym {
        use super::*;

        #[test]
        fn test_pool_build_nym_request() {
            let pool = TestPool::new();

            let nym_request =
                pool.request_builder()
                    .build_nym_request(&DidValue(String::from(TRUSTEE_DID)),
                                       &DidValue(String::from(MY1_DID)),
                                       None,
                                       None,
                                       None).unwrap();

            let expected_result = json!({
                "type": constants::NYM,
                "dest": MY1_DID,
            });

            helpers::check_request_operation(&nym_request, expected_result);
        }

        #[test]
        fn test_pool_build_nym_request_for_optional_fields() {
            let pool = TestPool::new();

            let nym_request =
                pool.request_builder()
                    .build_nym_request(&DidValue(String::from(TRUSTEE_DID)),
                                       &DidValue(String::from(MY1_DID)),
                                       Some(MY1_VERKEY.to_string()),
                                       Some(ALIAS.to_string()),
                                       Some(ROLE.to_string())).unwrap();

            let expected_result = json!({
                "type": constants::NYM,
                "dest": MY1_DID,
                "verkey": MY1_VERKEY,
                "alias": ALIAS,
                "role": role_to_code(Some(String::from(ROLE))).unwrap(),
            });

            helpers::check_request_operation(&nym_request, expected_result);
        }

        #[test]
        fn test_pool_build_nym_request_for_empty_role() {
            let pool = TestPool::new();

            let nym_request =
                pool.request_builder()
                    .build_nym_request(&DidValue(String::from(TRUSTEE_DID)),
                                       &DidValue(String::from(MY1_DID)),
                                       None,
                                       None,
                                       Some(String::from(""))).unwrap();

            let expected_result = json!({
                "type": constants::NYM,
                "dest": MY1_DID,
                "role": serde_json::Value::Null,
            });

            helpers::check_request_operation(&nym_request, expected_result);
        }

        #[test]
        fn test_pool_build_nym_request_for_fully_qualified_dids() {
            let pool = TestPool::new();

            let nym_request =
                pool.request_builder()
                    .build_nym_request(&DidValue(String::from(TRUSTEE_DID_FQ)),
                                       &DidValue(String::from(MY1_DID_FQ)),
                                       None,
                                       None,
                                       None).unwrap();

            let expected_result = json!({
                "type": constants::NYM,
                "dest": MY1_DID,
            });

            helpers::check_request_operation(&nym_request, expected_result);
        }

        #[test]
        fn test_build_nym_request_works_for_invalid_role() {
            let pool = TestPool::new();
            let trustee = Identity::trustee();
            let new_identity = Identity::new(None);
            let role = "INALID_ROLE_ALIAS";

            let _err =
                pool.request_builder()
                    .build_nym_request(&trustee.did,
                                       &new_identity.did,
                                       Some(new_identity.verkey.to_string()),
                                       None,
                                       Some(role.to_string())).unwrap_err();
        }
    }

    mod get_nym {
        use super::*;

        #[test]
        fn test_pool_build_get_nym_request() {
            let pool = TestPool::new();

            let nym_request =
                pool.request_builder()
                    .build_get_nym_request(Some(&DidValue(String::from(TRUSTEE_DID))),
                                           &DidValue(String::from(MY1_DID))).unwrap();

            let expected_result = json!({
                "type": constants::GET_NYM,
                "dest": MY1_DID,
            });

            helpers::check_request_operation(&nym_request, expected_result);
        }

        #[test]
        fn test_pool_build_get_nym_request_for_qualified_dids() {
            let pool = TestPool::new();

            let nym_request =
                pool.request_builder()
                    .build_get_nym_request(Some(&DidValue(String::from(TRUSTEE_DID_FQ))),
                                           &DidValue(String::from(MY1_DID_FQ))).unwrap();

            let expected_result = json!({
                "type": constants::GET_NYM,
                "dest": MY1_DID,
            });

            helpers::check_request_operation(&nym_request, expected_result);
        }
    }
}

#[cfg(test)]
mod send_nym {
    use super::*;
    use crate::ledger::constants::ROLE_REMOVE;

    #[test]
    fn test_pool_send_nym_request() {
        let pool = TestPool::new();
        let trustee = Identity::trustee();
        let new_identity = Identity::new(None);

        // Send NYM
        let mut nym_request =
            pool.request_builder()
                .build_nym_request(&trustee.did,
                                   &new_identity.did,
                                   Some(new_identity.verkey.to_string()),
                                   None,
                                   None).unwrap();

        trustee.sign_request(&mut nym_request);

        let nym_response = pool.send_request(&nym_request).unwrap();

        // Get NYM
        let get_nym_request =
            pool.request_builder()
                .build_get_nym_request(None,
                                       &new_identity.did).unwrap();

        let response = pool.send_request_with_retries(&get_nym_request, &nym_response).unwrap();

        let expected_data = json!({
            "dest": &new_identity.did,
            "verkey": &new_identity.verkey,
            "role": serde_json::Value::Null
        });
        assert_eq!(expected_data, parse_get_nym_response(&response));
    }

    #[test]
    fn test_pool_send_nym_request_for_optional_fields() {
        let pool = TestPool::new();
        let trustee = Identity::trustee();
        let new_identity = Identity::new(None);

        // Send NYM
        let mut nym_request =
            pool.request_builder()
                .build_nym_request(&trustee.did,
                                   &new_identity.did,
                                   Some(new_identity.verkey.to_string()),
                                   Some(ALIAS.to_string()),
                                   Some(ROLE.to_string())).unwrap();

        trustee.sign_request(&mut nym_request);

        let nym_response = pool.send_request(&nym_request).unwrap();

        // Get NYM
        let get_nym_request =
            pool.request_builder()
                .build_get_nym_request(None,
                                       &new_identity.did).unwrap();

        let response = pool.send_request_with_retries(&get_nym_request, &nym_response).unwrap();

        let expected_data = json!({
            "dest": &new_identity.did,
            "verkey": &new_identity.verkey,
            "role": role_to_code(Some(String::from(ROLE))).unwrap(),
        });
        assert_eq!(expected_data, parse_get_nym_response(&response));
    }

    #[test]
    fn test_pool_send_nym_request_for_different_roles() {
        let pool = TestPool::new();
        let trustee = Identity::trustee();

        for role in ["STEWARD", "TRUSTEE", "TRUST_ANCHOR", "ENDORSER", "NETWORK_MONITOR"].iter() {
            let new_identity = Identity::new(None);

            // Send NYM
            let mut nym_request =
                pool.request_builder()
                    .build_nym_request(&trustee.did,
                                       &new_identity.did,
                                       Some(new_identity.verkey.to_string()),
                                       None,
                                       Some(role.to_string())).unwrap();

            trustee.sign_request(&mut nym_request);

            let nym_response = pool.send_request(&nym_request).unwrap();

            // Get NYM
            let get_nym_request =
                pool.request_builder()
                    .build_get_nym_request(None,
                                           &new_identity.did).unwrap();

            let response = pool.send_request_with_retries(&get_nym_request, &nym_response).unwrap();

            let expected_data = json!({
                "dest": &new_identity.did,
                "verkey": &new_identity.verkey,
                "role": role_to_code(Some(role.to_string())).unwrap(),
            });
            assert_eq!(expected_data, parse_get_nym_response(&response));
        }
    }

    #[test]
    fn test_pool_send_nym_request_for_resetting_role() {
        let pool = TestPool::new();
        let trustee = Identity::trustee();
        let new_identity = Identity::new(None);

        // Send NYM with TRUSTEE role
        let mut nym_request =
            pool.request_builder()
                .build_nym_request(&trustee.did,
                                   &new_identity.did,
                                   Some(new_identity.verkey.to_string()),
                                   None,
                                   Some(ROLE.to_string())).unwrap();

        trustee.sign_request(&mut nym_request);

        let nym_response = pool.send_request(&nym_request).unwrap();

        // Get NYM to ensure role is TRUSTEE
        let get_nym_request =
            pool.request_builder()
                .build_get_nym_request(None,
                                       &new_identity.did).unwrap();

        let response = pool.send_request_with_retries(&get_nym_request, &nym_response).unwrap();

        let expected_data = json!({
            "dest": &new_identity.did,
            "verkey": &new_identity.verkey,
            "role": role_to_code(Some(String::from(ROLE))).unwrap(),
        });
        assert_eq!(expected_data, parse_get_nym_response(&response));

        // Send NYM with empty role to reset current
        let mut nym_request =
            pool.request_builder()
                .build_nym_request(&trustee.did,
                                   &new_identity.did,
                                   None,
                                   None,
                                   Some(ROLE_REMOVE.to_string())).unwrap();

        trustee.sign_request(&mut nym_request);

        let nym_response = pool.send_request(&nym_request).unwrap();

        // Get NYM to ensure role was reset
        let get_nym_request =
            pool.request_builder()
                .build_get_nym_request(None,
                                       &new_identity.did).unwrap();

        let response = pool.send_request_with_retries(&get_nym_request, &nym_response).unwrap();

        let expected_data = json!({
            "dest": &new_identity.did,
            "verkey": &new_identity.verkey,
            "role": serde_json::Value::Null,
        });
        assert_eq!(expected_data, parse_get_nym_response(&response));
    }

    #[test]
    fn test_pool_send_nym_request_without_signature() {
        let pool = TestPool::new();
        let trustee = Identity::trustee();
        let new_identity = Identity::new(None);

        // Send NYM
        let nym_request =
            pool.request_builder()
                .build_nym_request(&trustee.did,
                                   &new_identity.did,
                                   Some(new_identity.verkey.to_string()),
                                   None,
                                   None).unwrap();

        let err = pool.send_request(&nym_request).unwrap_err();
        helpers::check_response_type(&err, "REQNACK");
    }

    #[test]
    fn test_pool_send_nym_request_for_unknown_signer() {
        let pool = TestPool::new();
        let new_identity = Identity::new(None);

        // Send NYM
        let mut nym_request =
            pool.request_builder()
                .build_nym_request(&new_identity.did,
                                   &new_identity.did,
                                   None,
                                   None,
                                   None).unwrap();

        new_identity.sign_request(&mut nym_request);

        let err = pool.send_request(&nym_request).unwrap_err();
        helpers::check_response_type(&err, "REQNACK");
    }

    #[test]
    fn test_pool_send_get_nym_request_for_unknown_target() {
        let pool = TestPool::new();
        let new_identity = Identity::new(None);

        // Get NYM
        let get_nym_request =
            pool.request_builder()
                .build_get_nym_request(None,
                                       &new_identity.did).unwrap();

        let response = pool.send_request(&get_nym_request).unwrap();

        helpers::get_response_data(&response).unwrap_err();
    }

    fn parse_get_nym_response(response: &str) -> serde_json::Value {
        let data = helpers::get_response_data(response).unwrap();
        let data: serde_json::Value = serde_json::from_str(&data.as_str().unwrap()).unwrap();
        json!({
            "dest": data["dest"],
            "verkey": data["verkey"],
            "role": data["role"],
        })
    }
}