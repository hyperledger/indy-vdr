#[macro_use]
mod utils;

inject_dependencies!();

use indy_vdr::ledger::constants;
use indy_vdr::ledger::identifiers::{CredentialDefinitionId, RevocationRegistryId};
use indy_vdr::ledger::requests::rev_reg_def::RevocationRegistryDefinition;
use indy_vdr::utils::did::DidValue;

use crate::utils::fixtures::*;
use crate::utils::helpers;

#[test]
fn empty() {
    // Empty test to run module
}

#[cfg(test)]
mod builder {
    use super::*;
    use crate::utils::helpers::revoc_reg::*;
    use indy_vdr::ledger::RequestBuilder;

    fn _did() -> DidValue {
        DidValue("NcYxiDXkpYi6ov5FcYDi1e".to_string())
    }

    fn _cred_def_id() -> CredentialDefinitionId {
        CredentialDefinitionId("NcYxiDXkpYi6ov5FcYDi1e:3:CL:1:tag".to_string())
    }

    fn _rev_reg_id() -> RevocationRegistryId {
        RevocationRegistryId(
            "NcYxiDXkpYi6ov5FcYDi1e:4:NcYxiDXkpYi6ov5FcYDi1e:3:CL:1:tag:CL_ACCUM:tag".to_string(),
        )
    }

    mod revoc_reg_def {
        use super::*;

        fn _rev_reg_def() -> RevocationRegistryDefinition {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(build(
                &_did(),
                &_cred_def_id(),
            ))
        }

        #[rstest]
        fn test_build_revoc_reg_def_request(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_revoc_reg_def_request(&trustee_did, _rev_reg_def())
                .unwrap();

            let expected_operation = json!({
                "id": _rev_reg_id(),
                "credDefId": _cred_def_id(),
                "revocDefType": REVOC_DEF_TYPE,
                "tag": TAG,
                "type": constants::REVOC_REG_DEF,
                "value": revoc_reg_def_value()
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }

    mod get_revoc_reg_def {
        use super::*;

        #[rstest]
        fn test_build_get_revoc_reg_def_request(request_builder: RequestBuilder) {
            let request = request_builder
                .build_get_revoc_reg_def_request(None, &_rev_reg_id())
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_REVOC_REG_DEF,
                "id": _rev_reg_id()
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }

    mod revoc_reg_entry {
        use super::*;

        #[rstest]
        fn test_build_revoc_reg_entry_request(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_revoc_reg_entry_request(
                    &trustee_did,
                    &_rev_reg_id(),
                    &REVOC_DEF_TYPE,
                    revoc_reg_delta(),
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::REVOC_REG_ENTRY,
                "revocRegDefId": _rev_reg_id(),
                "revocDefType": REVOC_DEF_TYPE,
                "value": revoc_reg_entry_value()
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }

    mod get_revoc_reg {
        use super::*;

        #[rstest]
        fn test_build_get_revoc_reg_delta_request(request_builder: RequestBuilder) {
            let request = request_builder
                .build_get_revoc_reg_request(None, &_rev_reg_id(), TO)
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_REVOC_REG,
                "revocRegDefId": _rev_reg_id(),
                "timestamp": TO
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }

    mod get_revoc_reg_delta {
        use super::*;

        #[rstest]
        fn test_build_get_revoc_reg_delta_request(request_builder: RequestBuilder) {
            let request = request_builder
                .build_get_revoc_reg_delta_request(None, &_rev_reg_id(), None, TO)
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_REVOC_REG_DELTA,
                "revocRegDefId": _rev_reg_id(),
                "to": TO
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_get_revoc_reg_delta_request_for_both_timestamps(
            request_builder: RequestBuilder,
        ) {
            let request = request_builder
                .build_get_revoc_reg_delta_request(None, &_rev_reg_id(), Some(FROM), TO)
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_REVOC_REG_DELTA,
                "revocRegDefId": _rev_reg_id(),
                "from": FROM,
                "to": TO
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }
}

#[cfg(test)]
#[cfg(feature = "local_nodes_pool")]
mod send_revoc_reg {
    use super::*;

    use crate::helpers::revoc_reg::*;
    use crate::utils::pool::TestPool;

    #[rstest]
    fn test_pool_send_revoc_reg_def_def_requests(pool: TestPool) {
        let identity = helpers::new_ledger_identity(&pool, Some(String::from("TRUSTEE")));

        let schema = helpers::schema::default_schema(&identity.did);
        let (_schema_id, schema_seq_no) = helpers::schema::publish(&identity, &pool, &schema);

        let cred_def = helpers::cred_def::build(&identity.did, schema_seq_no);
        let cred_def_id = helpers::cred_def::publish(&identity, &pool, cred_def);

        let revoc_reg_def = helpers::revoc_reg::build(&identity.did, &cred_def_id);
        let revoc_reg_id = revoc_reg_def.id.clone();

        // Send Revocation Registry Definition
        let mut revoc_reg_def = pool
            .request_builder()
            .build_revoc_reg_def_request(
                &identity.did,
                RevocationRegistryDefinition::RevocationRegistryDefinitionV1(revoc_reg_def),
            )
            .unwrap();

        let revoc_reg_def_response =
            helpers::sign_and_send_request(&identity, &pool, &mut revoc_reg_def).unwrap();

        // Get Revocation Registry Definition
        let get_revoc_reg_def_request = pool
            .request_builder()
            .build_get_revoc_reg_def_request(None, &revoc_reg_id)
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_revoc_reg_def_request, &revoc_reg_def_response)
            .unwrap();
        assert_eq!(
            json!(helpers::revoc_reg::build(&identity.did, &cred_def_id)),
            helpers::get_response_data(&response).unwrap()
        );

        // Send Revocation Registry Entry
        let mut revoc_reg_delta_request = pool
            .request_builder()
            .build_revoc_reg_entry_request(
                &identity.did,
                &revoc_reg_id,
                &REVOC_DEF_TYPE,
                revoc_reg_delta(),
            )
            .unwrap();

        let revoc_reg_entry_response =
            helpers::sign_and_send_request(&identity, &pool, &mut revoc_reg_delta_request).unwrap();

        _get_revocation_registry_cases(&pool, &revoc_reg_id, &revoc_reg_entry_response);
        _get_revocation_registry_delta_cases(&pool, &revoc_reg_id, &revoc_reg_entry_response);
    }

    fn _get_revocation_registry_cases(
        pool: &TestPool,
        revoc_reg_id: &RevocationRegistryId,
        revoc_reg_entry_response: &str,
    ) {
        let txn_time = _extract_txn_time_from_reply(revoc_reg_entry_response) as i64;

        // Send Get Revocation Registry for timestamp equal writing time
        let response =
            _get_revocation_registry(pool, revoc_reg_id, txn_time, revoc_reg_entry_response);
        assert_eq!(
            json!(revoc_reg_delta())["value"],
            helpers::get_response_data(&response).unwrap()["value"]
        );

        // Send Get Revocation Registry for timestamp > txn_time
        let response = _get_revocation_registry(
            pool,
            revoc_reg_id,
            txn_time + 1000,
            revoc_reg_entry_response,
        );
        assert_eq!(
            json!(revoc_reg_delta())["value"],
            helpers::get_response_data(&response).unwrap()["value"]
        );

        // Send Get Revocation Registry for timestamp < txn_time
        let response = _get_revocation_registry(
            pool,
            revoc_reg_id,
            txn_time - 1000,
            revoc_reg_entry_response,
        );
        helpers::get_response_data(&response).unwrap_err();
    }

    fn _get_revocation_registry_delta_cases(
        pool: &TestPool,
        revoc_reg_id: &RevocationRegistryId,
        revoc_reg_entry_response: &str,
    ) {
        let txn_time = _extract_txn_time_from_reply(revoc_reg_entry_response) as i64;

        // Send Get Revocation Registry Delta for to==txn_time
        let response = _get_revocation_registry_delta(
            pool,
            revoc_reg_id,
            None,
            txn_time,
            revoc_reg_entry_response,
        );
        let _data = helpers::get_response_data(&response).unwrap();

        // Send Get Revocation Registry Delta for to > txn_time
        let response = _get_revocation_registry_delta(
            pool,
            revoc_reg_id,
            None,
            txn_time + 1000,
            revoc_reg_entry_response,
        );
        let _data = helpers::get_response_data(&response).unwrap();

        // Send Get Revocation Registry Delta for to < txn_time
        let response = _get_revocation_registry_delta(
            pool,
            revoc_reg_id,
            None,
            txn_time - 1000,
            revoc_reg_entry_response,
        );
        let _data = helpers::get_response_data(&response).unwrap_err();

        // Send Get Revocation Registry Delta for from==to==txn_time
        let response = _get_revocation_registry_delta(
            pool,
            revoc_reg_id,
            Some(txn_time),
            txn_time,
            revoc_reg_entry_response,
        );
        let _data = helpers::get_response_data(&response).unwrap();

        // Send Get Revocation Registry Delta for from<txn_time<to
        let response = _get_revocation_registry_delta(
            pool,
            revoc_reg_id,
            Some(txn_time - 1000),
            txn_time + 1000,
            revoc_reg_entry_response,
        );
        let _data = helpers::get_response_data(&response).unwrap();

        // Send Get Revocation Registry Delta for txn_time<from<to
        let response = _get_revocation_registry_delta(
            pool,
            revoc_reg_id,
            Some(txn_time + 1000),
            txn_time + 5000,
            revoc_reg_entry_response,
        );
        let _data = helpers::get_response_data(&response).unwrap();
    }

    fn _get_revocation_registry(
        pool: &TestPool,
        revoc_reg_id: &RevocationRegistryId,
        timestamp: i64,
        revoc_reg_entry_response: &str,
    ) -> String {
        let get_revoc_reg = pool
            .request_builder()
            .build_get_revoc_reg_request(None, &revoc_reg_id, timestamp)
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_revoc_reg, revoc_reg_entry_response)
            .unwrap();
        response
    }

    fn _get_revocation_registry_delta(
        pool: &TestPool,
        revoc_reg_id: &RevocationRegistryId,
        from: Option<i64>,
        to: i64,
        revoc_reg_entry_response: &str,
    ) -> String {
        let get_revoc_reg_delta = pool
            .request_builder()
            .build_get_revoc_reg_delta_request(None, &revoc_reg_id, from, to)
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_revoc_reg_delta, revoc_reg_entry_response)
            .unwrap();
        response
    }

    fn _extract_txn_time_from_reply(reply: &str) -> u64 {
        let reply: serde_json::Value = serde_json::from_str(reply).unwrap();
        reply["result"]["txnMetadata"]["txnTime"].as_u64().unwrap()
    }
}
