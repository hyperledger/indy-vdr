#[macro_use]
mod utils;

inject_dependencies!();

use indy_vdr::ledger::constants;
use indy_vdr::ledger::identifiers::SchemaId;
use indy_vdr::ledger::requests::schema::Schema;
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
    use crate::utils::helpers::schema::*;
    use indy_vdr::ledger::RequestBuilder;

    fn _did() -> DidValue {
        DidValue("NcYxiDXkpYi6ov5FcYDi1e".to_string())
    }

    fn _schema_id() -> SchemaId {
        SchemaId("NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0".to_string())
    }

    fn _schema_id_qualified() -> SchemaId {
        SchemaId("schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0".to_string())
    }

    fn schema() -> Schema {
        Schema::SchemaV1(default_schema(&_did()))
    }

    mod schema {
        use super::*;

        #[rstest]
        fn test_build_schema_request(request_builder: RequestBuilder, trustee_did: DidValue) {
            let request = request_builder
                .build_schema_request(&trustee_did, schema())
                .unwrap();

            let expected_operation = json!({
                "type": constants::SCHEMA,
                "data": {
                    "name": NAME,
                    "version": VERSION,
                    "attr_names": attributes()
                },
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_schema_request_for_fully_qualified_dids(
            request_builder: RequestBuilder,
            fq_trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_schema_request(&fq_trustee_did, schema())
                .unwrap();

            let expected_operation = json!({
                "type": constants::SCHEMA,
                "data": {
                    "name": NAME,
                    "version": VERSION,
                    "attr_names": attributes()
                },
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }

    mod get_schema {
        use super::*;

        #[rstest]
        fn test_get_build_schema_request(request_builder: RequestBuilder) {
            let request = request_builder
                .build_get_schema_request(None, &_schema_id())
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_SCHEMA,
                "dest": _did(),
                "data": {
                    "name": NAME,
                    "version": VERSION,
                },
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_get_build_schema_request_for_fully_qualified_dids(request_builder: RequestBuilder) {
            let request = request_builder
                .build_get_schema_request(None, &_schema_id_qualified())
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_SCHEMA,
                "dest": _did(),
                "data": {
                    "name": NAME,
                    "version": VERSION,
                },
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }
}

#[cfg(test)]
#[cfg(feature = "local_nodes_pool")]
mod send_schema {
    use super::*;
    use crate::utils::pool::TestPool;

    #[rstest]
    fn test_pool_schema_requests(pool: TestPool) {
        let identity = helpers::new_ledger_identity(&pool, Some(String::from("TRUSTEE")));
        let schema = helpers::schema::default_schema(&identity.did);

        // Send Schema
        let mut schema_request = pool
            .request_builder()
            .build_schema_request(&identity.did, Schema::SchemaV1(schema.clone()))
            .unwrap();

        let schema_response =
            helpers::sign_and_send_request(&identity, &pool, &mut schema_request).unwrap();

        // Get Schema
        let get_schema_request = pool
            .request_builder()
            .build_get_schema_request(None, &schema.id)
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_schema_request, &schema_response)
            .unwrap();

        let expected_data = json!({
            "attr_names":  schema.attr_names,
            "name": schema.name,
            "version": schema.version
        });

        assert_eq!(
            expected_data,
            helpers::get_response_data(&response).unwrap()
        );
    }
}
