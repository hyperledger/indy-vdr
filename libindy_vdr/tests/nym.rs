#[macro_use]
mod utils;

inject_dependencies!();

use indy_vdr::ledger::constants;
use indy_vdr::ledger::constants::LedgerRole;
use indy_vdr::utils::did::DidValue;

use crate::utils::crypto::Identity;
use crate::utils::fixtures::*;
use crate::utils::helpers;

const ALIAS: &str = "alias";
const ROLE: LedgerRole = LedgerRole::Trustee;

#[test]
fn empty() {
    // Empty test to run module
}

#[cfg(test)]
mod builder {
    use super::*;
    use indy_vdr::ledger::RequestBuilder;

    mod nym {
        use indy_vdr::ledger::constants::UpdateRole;

        use super::*;

        #[rstest]
        fn test_build_nym_request(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
            my_did: DidValue,
        ) {
            let nym_request = request_builder
                .build_nym_request(&trustee_did, &my_did, None, None, None, None, None)
                .unwrap();

            let expected_result = json!({
                "type": constants::NYM,
                "dest": my_did,
            });

            helpers::check_request_operation(&nym_request, expected_result);
        }

        #[rstest]
        fn test_build_nym_request_for_optional_fields(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
            identity: Identity,
            diddoc_content: serde_json::Value,
        ) {
            // let copy = diddoc_content.clone();
            let nym_request = request_builder
                .build_nym_request(
                    &trustee_did,
                    &identity.did,
                    Some(identity.verkey.clone()),
                    Some(ALIAS.to_string()),
                    Some(UpdateRole::Set(ROLE)),
                    Some(&diddoc_content),
                    None,
                )
                .unwrap();

            let expected_result = json!({
                "type": constants::NYM,
                "dest": identity.did,
                "verkey": identity.verkey,
                "alias": ALIAS,
                "role": ROLE.to_code(),
                "diddocContent": diddoc_content.to_string(),
            });

            helpers::check_request_operation(&nym_request, expected_result);
        }

        #[rstest]
        fn test_pool_build_nym_request_for_empty_role(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
            my_did: DidValue,
        ) {
            let nym_request = request_builder
                .build_nym_request(
                    &trustee_did,
                    &my_did,
                    None,
                    None,
                    Some(UpdateRole::Reset),
                    None,
                    None,
                )
                .unwrap();

            let expected_result = json!({
                "type": constants::NYM,
                "dest": my_did,
                "role": serde_json::Value::Null,
            });

            helpers::check_request_operation(&nym_request, expected_result);
        }

        #[rstest]
        fn test_pool_build_nym_request_for_fully_qualified_dids(
            request_builder: RequestBuilder,
            fq_trustee_did: DidValue,
            fq_my_did: DidValue,
            my_did: DidValue,
        ) {
            let nym_request = request_builder
                .build_nym_request(&fq_trustee_did, &fq_my_did, None, None, None, None, None)
                .unwrap();

            let expected_result = json!({
                "type": constants::NYM,
                "dest": my_did,
            });

            helpers::check_request_operation(&nym_request, expected_result);
        }
    }

    mod get_nym {
        use super::*;

        #[rstest]
        fn test_pool_build_get_nym_request(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
            my_did: DidValue,
        ) {
            let nym_request = request_builder
                .build_get_nym_request(Some(&trustee_did), &my_did, None, None)
                .unwrap();

            let expected_result = json!({
                "type": constants::GET_NYM,
                "dest": my_did,
            });

            helpers::check_request_operation(&nym_request, expected_result);
        }

        #[rstest]
        fn test_pool_build_get_nym_request_for_qualified_dids(
            request_builder: RequestBuilder,
            fq_trustee_did: DidValue,
            fq_my_did: DidValue,
            my_did: DidValue,
        ) {
            let nym_request = request_builder
                .build_get_nym_request(Some(&fq_trustee_did), &fq_my_did, None, None)
                .unwrap();

            let expected_result = json!({
                "type": constants::GET_NYM,
                "dest": my_did,
            });

            helpers::check_request_operation(&nym_request, expected_result);
        }
    }
}

#[cfg(test)]
#[cfg(feature = "local_nodes_pool")]
mod send {
    use super::*;
    use crate::utils::pool::TestPool;
    use indy_vdr::ledger::{constants::UpdateRole, responses::GetNymResultV1};
    use std::str::FromStr;

    #[rstest]
    fn test_pool_send_nym_request(pool: TestPool, trustee: Identity, identity: Identity) {
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

        // Get NYM
        let get_nym_request = pool
            .request_builder()
            .build_get_nym_request(None, &identity.did, None, None)
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_nym_request, &nym_response)
            .unwrap();

        let expected_data = json!({
            "dest": &identity.did,
            "verkey": &identity.verkey,
            "role": serde_json::Value::Null
        });
        assert_eq!(expected_data, parse_get_nym_response(&response));
    }

    #[rstest]
    fn test_pool_send_nym_request_for_optional_fields(
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
                Some(ALIAS.to_string()),
                Some(UpdateRole::Set(ROLE)),
                None,
                None,
            )
            .unwrap();

        let nym_response =
            helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap();

        // Get NYM
        let get_nym_request = pool
            .request_builder()
            .build_get_nym_request(None, &identity.did, None, None)
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_nym_request, &nym_response)
            .unwrap();

        let expected_data = json!({
            "dest": &identity.did,
            "verkey": &identity.verkey,
            "role": ROLE.to_code(),
        });
        assert_eq!(expected_data, parse_get_nym_response(&response));
    }

    #[rstest]
    fn test_pool_send_nym_request_with_diddoc_content(
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
                Some(ALIAS.to_string()),
                Some(UpdateRole::Set(ROLE)),
                Some(&diddoc_content),
                None,
            )
            .unwrap();

        let nym_response =
            helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap();

        // Get NYM
        let get_nym_request = pool
            .request_builder()
            .build_get_nym_request(None, &identity.did, None, None)
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_nym_request, &nym_response)
            .unwrap();

        let expected_data = json!({
            "identifier": &trustee.did,
            "dest": &identity.did,
            "verkey": &identity.verkey,
            "role": ROLE.to_code(),
            "diddocContent": &diddoc_content.to_string()
        });

        let data: GetNymResultV1 = serde_json::from_str(
            helpers::get_response_data(&response)
                .unwrap()
                .as_str()
                .unwrap(),
        )
        .unwrap();

        assert_eq!(expected_data, serde_json::to_value(data).unwrap());
    }

    #[rstest]
    fn test_pool_send_nym_request_with_version_2_fails(
        pool: TestPool,
        trustee: Identity,
        identity: Identity,
    ) {
        // Send NYM
        let mut nym_request = pool
            .request_builder()
            .build_nym_request(
                &trustee.did,
                &identity.did, // Self-cert version 1 identifier
                Some(identity.verkey.to_string()),
                Some(ALIAS.to_string()),
                Some(UpdateRole::Set(ROLE)),
                None,
                Some(2), // Claim Self-cert version 2
            )
            .unwrap();

        let _err = helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap_err();
    }

    #[rstest]
    fn test_pool_send_nym_request_with_non_self_cert_did_fails(
        pool: TestPool,
        trustee: Identity,
        non_self_cert_identity: Identity,
    ) {
        // Send NYM
        let identity = non_self_cert_identity;
        let mut nym_request = pool
            .request_builder()
            .build_nym_request(
                &trustee.did,
                &identity.did, // Non self-cert identifier
                Some(identity.verkey.to_string()),
                Some(ALIAS.to_string()),
                Some(UpdateRole::Set(ROLE)),
                None,
                Some(2), // Claim Self-cert version 2
            )
            .unwrap();

        let _err = helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap_err();
    }

    #[rstest]
    fn test_pool_send_nym_request_with_non_self_cert_did_works(
        pool: TestPool,
        trustee: Identity,
        non_self_cert_identity: Identity,
    ) {
        // Send NYM
        let identity = non_self_cert_identity;
        println!("DID: {}", identity.did);
        let mut nym_request = pool
            .request_builder()
            .build_nym_request(
                &trustee.did,
                &identity.did, // Non self-cert identifier
                Some(identity.verkey.to_string()),
                Some(ALIAS.to_string()),
                Some(UpdateRole::Set(ROLE)),
                None,
                None,
            )
            .unwrap();

        let _err = helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap();
    }

    #[rstest]
    fn test_pool_send_nym_request_with_version_0_works(
        pool: TestPool,
        trustee: Identity,
        identity: Identity,
    ) {
        // Send NYM
        let mut nym_request = pool
            .request_builder()
            .build_nym_request(
                &trustee.did,
                &identity.did, // Self-cert version 1 identifier
                Some(identity.verkey.to_string()),
                Some(ALIAS.to_string()),
                Some(UpdateRole::Set(ROLE)),
                None,
                Some(0), // Claim Self-cert version 0
            )
            .unwrap();

        let nym_response =
            helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap();

        // Get NYM
        let get_nym_request = pool
            .request_builder()
            .build_get_nym_request(None, &identity.did, None, None)
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_nym_request, &nym_response)
            .unwrap();

        let expected_data = json!({
            "identifier": &trustee.did,
            "dest": &identity.did,
            "verkey": &identity.verkey,
            "role": ROLE.to_code(),
            "version": 0,
        });

        let data: GetNymResultV1 = serde_json::from_str(
            helpers::get_response_data(&response)
                .unwrap()
                .as_str()
                .unwrap(),
        )
        .unwrap();

        assert_eq!(expected_data, serde_json::to_value(data).unwrap());
    }

    #[rstest]
    fn test_pool_send_nym_request_with_version_2_works(
        pool: TestPool,
        trustee: Identity,
        identity_v2: Identity,
        diddoc_content: serde_json::Value,
    ) {
        // Send NYM
        let identity = identity_v2;
        let mut nym_request = pool
            .request_builder()
            .build_nym_request(
                &trustee.did,
                &identity.did,
                Some(identity.verkey.to_string()),
                Some(ALIAS.to_string()),
                Some(UpdateRole::Set(ROLE)),
                Some(&diddoc_content),
                Some(2),
            )
            .unwrap();

        let nym_response =
            helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap();

        // Get NYM
        let get_nym_request = pool
            .request_builder()
            .build_get_nym_request(None, &identity.did, None, None)
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_nym_request, &nym_response)
            .unwrap();

        let expected_data = json!({
            "identifier": &trustee.did,
            "dest": &identity.did,
            "verkey": &identity.verkey,
            "role": ROLE.to_code(),
            "diddocContent": &diddoc_content.to_string(),
            "version": 2,
        });

        let data: GetNymResultV1 = serde_json::from_str(
            helpers::get_response_data(&response)
                .unwrap()
                .as_str()
                .unwrap(),
        )
        .unwrap();

        assert_eq!(expected_data, serde_json::to_value(data).unwrap());
    }

    #[rstest(
        role,
        case("STEWARD"),
        case("TRUSTEE"),
        case("TRUST_ANCHOR"),
        case("ENDORSER"),
        case("NETWORK_MONITOR")
    )]
    fn test_pool_send_nym_request_for_different_roles(
        pool: TestPool,
        trustee: Identity,
        role: &str,
    ) {
        let new_identity = Identity::new(None, None);
        let upd_role = UpdateRole::from_str(role).unwrap();

        // Send NYM
        let mut nym_request = pool
            .request_builder()
            .build_nym_request(
                &trustee.did,
                &new_identity.did,
                Some(new_identity.verkey.to_string()),
                None,
                Some(upd_role),
                None,
                None,
            )
            .unwrap();

        let nym_response =
            helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap();

        // Get NYM
        let get_nym_request = pool
            .request_builder()
            .build_get_nym_request(None, &new_identity.did, None, None)
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_nym_request, &nym_response)
            .unwrap();

        let expected_data = json!({
            "dest": &new_identity.did,
            "verkey": &new_identity.verkey,
            "role": LedgerRole::from_str(role).unwrap().to_code(),
        });
        assert_eq!(expected_data, parse_get_nym_response(&response));
    }

    #[rstest]
    fn test_pool_send_nym_request_for_resetting_role(
        pool: TestPool,
        trustee: Identity,
        identity: Identity,
    ) {
        // Send NYM with TRUSTEE role
        let mut nym_request = pool
            .request_builder()
            .build_nym_request(
                &trustee.did,
                &identity.did,
                Some(identity.verkey.to_string()),
                None,
                Some(UpdateRole::Set(ROLE)),
                None,
                None,
            )
            .unwrap();

        let nym_response =
            helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap();

        // Get NYM to ensure role is TRUSTEE
        let get_nym_request = pool
            .request_builder()
            .build_get_nym_request(None, &identity.did, None, None)
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_nym_request, &nym_response)
            .unwrap();

        let expected_data = json!({
            "dest": &identity.did,
            "verkey": &identity.verkey,
            "role": ROLE.to_code(),
        });
        assert_eq!(expected_data, parse_get_nym_response(&response));

        // Send NYM with empty role to reset current
        let mut nym_request = pool
            .request_builder()
            .build_nym_request(
                &trustee.did,
                &identity.did,
                None,
                None,
                Some(UpdateRole::Reset),
                None,
                None,
            )
            .unwrap();

        let nym_response =
            helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap();

        // Get NYM to ensure role was reset
        let get_nym_request = pool
            .request_builder()
            .build_get_nym_request(None, &identity.did, None, None)
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_nym_request, &nym_response)
            .unwrap();

        let expected_data = json!({
            "dest": &identity.did,
            "verkey": &identity.verkey,
            "role": serde_json::Value::Null,
        });
        assert_eq!(expected_data, parse_get_nym_response(&response));
    }

    #[rstest]
    fn test_pool_send_nym_request_without_signature(
        pool: TestPool,
        trustee: Identity,
        identity: Identity,
    ) {
        // Send NYM
        let nym_request = pool
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

        let err = pool.send_request(&nym_request).unwrap_err();
        helpers::check_response_type(&err, "REQNACK");
    }

    #[rstest]
    fn test_pool_send_nym_request_for_unknown_signer(pool: TestPool, identity: Identity) {
        // Send NYM
        let mut nym_request = pool
            .request_builder()
            .build_nym_request(&identity.did, &identity.did, None, None, None, None, None)
            .unwrap();

        identity.sign_request(&mut nym_request);

        let err = pool.send_request(&nym_request).unwrap_err();
        helpers::check_response_type(&err, "REQNACK");
    }

    #[rstest]
    fn test_pool_send_nym_request_for_wrong_signer_role(pool: TestPool) {
        let identity = helpers::new_ledger_identity(&pool, None);
        let new_identity = Identity::new(None, None);

        // Send NYM
        let mut nym_request = pool
            .request_builder()
            .build_nym_request(
                &identity.did,
                &new_identity.did,
                Some(new_identity.verkey),
                None,
                None,
                None,
                None,
            )
            .unwrap();

        let err = helpers::sign_and_send_request(&identity, &pool, &mut nym_request).unwrap_err();
        helpers::check_response_type(&err, "REJECT");
    }

    #[rstest]
    fn test_pool_send_get_nym_request_for_unknown_target(pool: TestPool, identity: Identity) {
        // Get NYM
        let get_nym_request = pool
            .request_builder()
            .build_get_nym_request(None, &identity.did, None, None)
            .unwrap();

        let response = pool.send_request(&get_nym_request).unwrap();

        helpers::get_response_data(&response).unwrap_err();
    }

    fn parse_get_nym_response(response: &str) -> serde_json::Value {
        let data = helpers::get_response_data(response).unwrap();
        let data: serde_json::Value = serde_json::from_str(data.as_str().unwrap()).unwrap();
        json!({
            "dest": data["dest"],
            "verkey": data["verkey"],
            "role": data["role"],
        })
    }
}
