#[macro_use]
mod utils;

inject_dependencies!();

use indy_vdr::common::did::DidValue;
use indy_vdr::ledger::constants;

use crate::utils::helpers;
use crate::utils::fixtures::*;
use crate::utils::pool::*;
use indy_vdr::ledger::identifiers::schema::SchemaId;
use indy_vdr::ledger::requests::schema::{Schema, SchemaV1, AttributeNames};
use std::collections::HashSet;

fn _did() -> DidValue {
    DidValue("NcYxiDXkpYi6ov5FcYDi1e".to_string())
}

fn _name() -> String { String::from("gvt") }

fn _version() -> String { String::from("1.0") }

fn _attributes() -> AttributeNames {
    let mut attributes = HashSet::new();
    attributes.insert(String::from("name"));
    AttributeNames(attributes)
}

fn _schema_id() -> SchemaId {
    SchemaId("NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0".to_string())
}

fn _schema_id_qualified() -> SchemaId {
    SchemaId("schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0".to_string())
}

fn _build_schema_id(did: &DidValue, name: &str, version: &str) -> SchemaId {
    SchemaId(format!("{}:2:{}:{}", did.0, name, version))
}

fn _schema() -> Schema {
    Schema::SchemaV1(SchemaV1 {
        id: _schema_id(),
        name: _name(),
        version: _version(),
        attr_names: _attributes(),
        seq_no: None,
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

    mod schema {
        use super::*;

        #[rstest]
        fn test_build_schema_request(request_builder: RequestBuilder,
                                     trustee_did: DidValue) {
            let request =
                request_builder
                    .build_schema_request(&trustee_did,
                                          _schema()).unwrap();

            let expected_operation = json!({
                "type": constants::SCHEMA,
                "data": {
                    "name": _name(),
                    "version": _version(),
                    "attr_names": _attributes()
                },
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_schema_request_for_fully_qualified_dids(request_builder: RequestBuilder,
                                                              fq_trustee_did: DidValue) {
            let request =
                request_builder
                    .build_schema_request(&fq_trustee_did,
                                          _schema()).unwrap();

            let expected_operation = json!({
                "type": constants::SCHEMA,
                "data": {
                    "name": _name(),
                    "version": _version(),
                    "attr_names": _attributes()
                },
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }

    mod get_schema {
        use super::*;

        #[rstest]
        fn test_get_build_schema_request(request_builder: RequestBuilder) {
            let request =
                request_builder
                    .build_get_schema_request(None,
                                              &_schema_id()).unwrap();

            let expected_operation = json!({
                "type": constants::GET_SCHEMA,
                "dest": _did(),
                "data": {
                    "name": _name(),
                    "version": _version(),
                },
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_get_build_schema_request_for_fully_qualified_dids(request_builder: RequestBuilder) {
            let request =
                request_builder
                    .build_get_schema_request(None,
                                              &_schema_id_qualified()).unwrap();

            let expected_operation = json!({
                "type": constants::GET_SCHEMA,
                "dest": _did(),
                "data": {
                    "name": _name(),
                    "version": _version(),
                },
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }
}

#[cfg(test)]
mod send_schema {
    use super::*;

    #[rstest]
    fn test_pool_schema_requests(pool: TestPool) {
        let identity = helpers::new_ledger_identity(&pool, Some(String::from("TRUSTEE")));

        // Send Schema
        let mut schema_request =
            pool.request_builder()
                .build_schema_request(&identity.did,
                                      _schema()).unwrap();

        let schema_response = helpers::sign_and_send_request(&identity, &pool, &mut schema_request).unwrap();

        let schema_id = _build_schema_id(&identity.did, &_name(), &_version());

        // Get Schema
        let get_schema_request =
            pool.request_builder()
                .build_get_schema_request(None,
                                          &schema_id).unwrap();

        let response = pool.send_request_with_retries(&get_schema_request, &schema_response).unwrap();

        let expected_data = json!({
            "attr_names":  _attributes(),
            "name": _name(),
            "version": _version()
        });

        assert_eq!(expected_data, helpers::get_response_data(&response).unwrap());
    }
}