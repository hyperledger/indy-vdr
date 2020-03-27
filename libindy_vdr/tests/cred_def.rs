#[macro_use]
mod utils;

inject_dependencies!();

use indy_vdr::common::did::DidValue;
use indy_vdr::ledger::constants;

use crate::utils::helpers;
use crate::utils::fixtures::*;
use indy_vdr::ledger::identifiers::cred_def::CredentialDefinitionId;
use indy_vdr::ledger::identifiers::schema::SchemaId;
use indy_vdr::ledger::requests::cred_def::{CredentialDefinition, CredentialDefinitionV1, SignatureType};

const SCHEMA_SEQ_NO: i32 = 1;
const TAG: &'static str = "tag";
const TYPE: SignatureType = SignatureType::CL;


fn _did() -> DidValue {
    DidValue("NcYxiDXkpYi6ov5FcYDi1e".to_string())
}

fn _schema_id() -> SchemaId {
    SchemaId(SCHEMA_SEQ_NO.to_string())
}

fn _cred_def_id() -> CredentialDefinitionId {
    CredentialDefinitionId("NcYxiDXkpYi6ov5FcYDi1e:3:CL:1:tag".to_string())
}

fn _cred_def_data() -> serde_json::Value {
    json!({
        "primary":{
           "n":"1",
           "s":"2",
           "r":{"name":"1","master_secret":"3"},
           "rctxt":"1",
           "z":"1"
        }
    })
}

fn _cred_def() -> CredentialDefinition {
    CredentialDefinition::CredentialDefinitionV1(CredentialDefinitionV1 {
        id: _cred_def_id(),
        schema_id: _schema_id(),
        signature_type: TYPE,
        tag: TAG.to_string(),
        value: serde_json::from_value(_cred_def_data()).unwrap(),
    })
}

#[test]
fn empty() {
    // Empty test to run module
}

#[cfg(test)]
mod builder {
    use super::*;
    use indy_vdr::ledger::RequestBuilder;

    mod cred_def {
        use super::*;

        #[rstest]
        fn test_build_cred_def_request(request_builder: RequestBuilder,
                                       trustee_did: DidValue) {
            let request =
                request_builder
                    .build_cred_def_request(&trustee_did,
                                            _cred_def()).unwrap();

            let expected_operation = json!({
                "ref": 1,
                "type": constants::CRED_DEF,
                "signature_type": TYPE,
                "tag": TAG,
                "data": _cred_def_data()
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }

    mod get_cred_def {
        use super::*;

        #[rstest]
        fn test_build_get_cred_def_request(request_builder: RequestBuilder) {
            let request =
                request_builder
                    .build_get_cred_def_request(None,
                                                &_cred_def_id()).unwrap();

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