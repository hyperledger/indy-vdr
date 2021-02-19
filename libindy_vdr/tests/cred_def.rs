#[macro_use]
mod utils;

inject_dependencies!();

use indy_vdr::ledger::constants;
use indy_vdr::ledger::identifiers::{CredentialDefinitionId, SchemaId};
use indy_vdr::ledger::requests::cred_def::CredentialDefinition;
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
    use crate::utils::helpers::cred_def::*;
    use indy_vdr::ledger::RequestBuilder;

    fn _did() -> DidValue {
        DidValue("NcYxiDXkpYi6ov5FcYDi1e".to_string())
    }

    fn _schema_id() -> SchemaId {
        SchemaId(SCHEMA_SEQ_NO.to_string())
    }

    fn _cred_def_id() -> CredentialDefinitionId {
        CredentialDefinitionId("NcYxiDXkpYi6ov5FcYDi1e:3:CL:1:tag".to_string())
    }

    fn _cred_def() -> CredentialDefinition {
        CredentialDefinition::CredentialDefinitionV1(build(&_did(), SCHEMA_SEQ_NO))
    }

    mod cred_def {
        use super::*;

        #[rstest]
        fn test_build_cred_def_request(request_builder: RequestBuilder, trustee_did: DidValue) {
            let request = request_builder
                .build_cred_def_request(&trustee_did, _cred_def())
                .unwrap();

            let expected_operation = json!({
                "ref": 1,
                "type": constants::CRED_DEF,
                "signature_type": TYPE,
                "tag": TAG,
                "data": cred_def_data()
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }

    mod get_cred_def {
        use super::*;

        #[rstest]
        fn test_build_get_cred_def_request(request_builder: RequestBuilder) {
            let request = request_builder
                .build_get_cred_def_request(None, &_cred_def_id())
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_CRED_DEF,
                "ref": SCHEMA_SEQ_NO,
                "signature_type": TYPE,
                "origin": _did(),
                "tag": TAG
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }
}

#[cfg(test)]
#[cfg(feature = "local_nodes_pool")]
mod send_cred_def {
    use super::*;
    use crate::utils::pool::TestPool;

    #[rstest]
    fn test_pool_send_cred_def_requests(pool: TestPool) {
        let identity = helpers::new_ledger_identity(&pool, Some(String::from("TRUSTEE")));

        let schema = helpers::schema::default_schema(&identity.did);
        let (_schema_id, schema_seq_no) = helpers::schema::publish(&identity, &pool, &schema);

        let cred_def = helpers::cred_def::build(&identity.did, schema_seq_no);
        let cred_def_id = cred_def.id.clone();

        // Send Credential Definition
        let mut cred_def_request = pool
            .request_builder()
            .build_cred_def_request(
                &identity.did,
                CredentialDefinition::CredentialDefinitionV1(cred_def),
            )
            .unwrap();

        let cred_def_response =
            helpers::sign_and_send_request(&identity, &pool, &mut cred_def_request).unwrap();

        // Get Credential Definition
        let get_cred_def_request = pool
            .request_builder()
            .build_get_cred_def_request(None, &cred_def_id)
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_cred_def_request, &cred_def_response)
            .unwrap();
        assert_eq!(
            helpers::cred_def::cred_def_data(),
            helpers::get_response_data(&response).unwrap()
        )
    }
}
