extern crate rand;
use crate::tests::utils::pool::*;
use crate::common::did::DidValue;
use crate::ledger::identifiers::rich_schema::RichSchemaId;
use crate::ledger::constants;
use crate::tests::utils::constants::{TRUSTEE_DID, TRUSTEE_DID_FQ};
use crate::tests::utils::helpers;
use rand::Rng;


#[cfg(test)]
mod builder {
    use super::*;
    use std;
    use crate::ledger::requests::rich_schema::{RichSchema, RSContent};
    use crate::ledger::PreparedRequest;
    use crate::utils::test::get_rand_string;

    // const RS_CONTENT_STRING: &str = r#"{"@id": "did:sov:some_hash_value", "json": "ld", "valid": "object", "@type": "sch"}"#;

    fn _rs_id_str() -> String {
        let mut id = "did:sov:".to_string();
        id.push_str(&get_rand_string(32));
        return id;
    }

    fn _rs_id(str_repr: String) -> RichSchemaId {
        return RichSchemaId::new(_rs_id_str());
    }

    fn _rs_version() -> String {
        let mut rng = rand::thread_rng();
        return format!("{}.{}.{}", rng.gen::<u32>(), rng.gen::<u32>(), rng.gen::<u32>()).to_string();
    }

    fn _rs_content(rs_id: RichSchemaId) -> RSContent {
        let rs_as_json = json!({
            "@id": rs_id,
            "@type": constants::RS_SCHEMA_TYPE_VALUE.to_string(),
            "some": "other".to_string(),
            "valid": "objects".to_string(),
        });
        return RSContent(rs_as_json.to_string());
    }

    pub(crate) fn _rich_schema() -> RichSchema {
        let rs_id = _rs_id(_rs_id_str());
        RichSchema::new(
            rs_id.clone(),
            _rs_content(rs_id),
            "test_rich_schema".to_string(),
            _rs_version(),
            constants::RS_SCHEMA_TYPE_VALUE.to_string(),
            "1".to_string(),
        )
    }

    pub fn _build_rs_req(identity: DidValue, rich_schema: RichSchema) -> PreparedRequest {
        let pool = TestPool::new();
        return pool.request_builder()
            .build_rich_schema_request(&identity, rich_schema)
            .unwrap();
    }

    #[test]
    fn test_rs_request_general(){
        let rich_schema = _rich_schema();
        let rs_req = _build_rs_req(DidValue(String::from(TRUSTEE_DID)), rich_schema.clone());
        let rs_id = _rs_id(_rs_id_str());
        let expected_result = json!({
                "type": constants::RICH_SCHEMA,
                "id": rich_schema.id,
                "content": _rs_content(rich_schema.id),
                "rsName": "test_rich_schema".to_string(),
                "rsVersion": rich_schema.rs_version,
                "rsType": constants::RS_SCHEMA_TYPE_VALUE.to_string(),
                "ver": "1".to_string()
            });
        helpers::check_request_operation(&rs_req, expected_result);
    }


    #[test]
    fn test_rs_request_works_for_fully_qualified_did(){
        let rich_schema = _rich_schema();
        let rs_req = _build_rs_req(DidValue(String::from(TRUSTEE_DID_FQ)), rich_schema.clone());
        let rs_id = _rs_id(_rs_id_str());
        let expected_result = json!({
                "type": constants::RICH_SCHEMA,
                "id": rich_schema.id,
                "content": _rs_content(rich_schema.id),
                "rsName": "test_rich_schema".to_string(),
                "rsVersion": rich_schema.rs_version,
                "rsType": constants::RS_SCHEMA_TYPE_VALUE.to_string(),
                "ver": "1".to_string()
            });
        helpers::check_request_operation(&rs_req, expected_result);
    }
}

mod sender{
    use super::*;
    use builder;
    use crate::tests::utils::crypto::Identity;

    fn _test_pool() -> TestPool {
        return TestPool::new();
    }

    #[test]
    fn test_rs_request_send_to_ledger() {
        let pool = _test_pool();
        let trustee = Identity::trustee();
        let rich_schema = builder::_rich_schema();
        let mut rs_req = builder::_build_rs_req(DidValue(String::from(TRUSTEE_DID)), rich_schema);
        trustee.sign_request(&mut rs_req);

        let rs_response = pool.send_request(&rs_req).unwrap();
        println!("{:?}", rs_response);
    }
}