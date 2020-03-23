extern crate rand;
use crate::common::did::DidValue;
use crate::ledger::constants;
use crate::ledger::identifiers::rich_schema::RichSchemaId;
use crate::tests::utils::constants::{TRUSTEE_DID, TRUSTEE_DID_FQ};
use crate::tests::utils::helpers;
use crate::tests::utils::pool::*;
use rand::Rng;

#[cfg(test)]
mod builder {
    use super::*;
    use crate::ledger::requests::rich_schema::{RSContent, RichSchema};
    use crate::ledger::PreparedRequest;
    use crate::utils::test::get_rand_string;

    fn _rs_id_str() -> String {
        let mut id = "did:sov:".to_string();
        id.push_str(&get_rand_string(32));
        return id;
    }

    pub(crate) fn _rs_id() -> RichSchemaId {
        return RichSchemaId::new(_rs_id_str());
    }

    fn _rs_version() -> String {
        let mut rng = rand::thread_rng();
        return format!(
            "{}.{}.{}",
            rng.gen::<u32>(),
            rng.gen::<u32>(),
            rng.gen::<u32>()
        )
        .to_string();
    }

    pub(crate) fn _rs_content(rs_id: RichSchemaId) -> RSContent {
        let rs_as_json = json!({
            "@id": rs_id,
            "@type": constants::RS_SCHEMA_TYPE_VALUE.to_string(),
            "some": "other".to_string(),
            "valid": "objects".to_string(),
        });
        return RSContent(rs_as_json.to_string());
    }

    pub(crate) fn _rich_schema() -> RichSchema {
        let rs_id = _rs_id();
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
        return pool
            .request_builder()
            .build_rich_schema_request(
                &identity,
                rich_schema.id,
                rich_schema.content,
                rich_schema.rs_name,
                rich_schema.rs_version,
                rich_schema.rs_type,
                rich_schema.ver,
            )
            .unwrap();
    }

    #[test]
    fn test_rs_request_general() {
        let rich_schema = _rich_schema();
        let rs_req = _build_rs_req(DidValue(String::from(TRUSTEE_DID)), rich_schema.clone());
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
    fn test_rs_request_works_for_fully_qualified_did() {
        let rich_schema = _rich_schema();
        let rs_req = _build_rs_req(DidValue(String::from(TRUSTEE_DID_FQ)), rich_schema.clone());
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

mod sender {
    use super::*;
    use crate::ledger::requests::rich_schema::{RSContent, RichSchema};
    use crate::tests::utils::crypto::Identity;
    use builder;
    use rstest::*;

    #[fixture]
    fn test_pool() -> TestPool {
        let t = TestPool::new();
        t
    }

    #[fixture]
    fn trustee() -> Identity {
        let t = Identity::trustee();
        t
    }

    #[rstest]
    fn test_rs_request_send_to_ledger_general(trustee: Identity, test_pool: TestPool) {
        let rich_schema = builder::_rich_schema();
        let rs_id = rich_schema.clone();
        let rs_meta = rich_schema.clone();
        let mut rs_req =
            builder::_build_rs_req(DidValue(String::from(TRUSTEE_DID)), rich_schema.clone());
        trustee.sign_request(&mut rs_req);

        let rs_response = test_pool.send_request(&rs_req).unwrap();
        let expected_result = json!({
            "id": rich_schema.id,
            "content": builder::_rs_content(rich_schema.id),
            "rsName": "test_rich_schema".to_string(),
            "rsVersion": rich_schema.rs_version,
            "rsType": constants::RS_SCHEMA_TYPE_VALUE.to_string(),
            "ver": "1".to_string(),
            "from": TRUSTEE_DID.to_string(),
            "endorser": serde_json::Value::Null,
        });

        let get_rs_by_id = test_pool
            .request_builder()
            .build_get_rich_schema_by_id(&DidValue(String::from(TRUSTEE_DID)), &rs_id.id)
            .unwrap();
        let response_by_id = test_pool
            .send_request_with_retries(&get_rs_by_id, &rs_response)
            .unwrap();

        let get_rs_by_metadata = test_pool
            .request_builder()
            .build_get_rich_schema_by_metadata(
                &DidValue(String::from(TRUSTEE_DID)),
                rs_meta.rs_type,
                rs_meta.rs_name,
                rs_meta.rs_version,
            )
            .unwrap();
        let response_by_metadata = test_pool
            .send_request_with_retries(&get_rs_by_metadata, &rs_response)
            .unwrap();

        assert_eq!(
            expected_result,
            helpers::get_response_data(&response_by_id).unwrap()
        );
        assert_eq!(
            expected_result,
            helpers::get_response_data(&response_by_metadata).unwrap()
        );
    }

    #[rstest]
    fn test_rs_request_send_to_ledger_with_already_exist_id(
        trustee: Identity,
        test_pool: TestPool,
    ) {
        let rich_schema = builder::_rich_schema();
        let mut rs_req_1 =
            builder::_build_rs_req(DidValue(String::from(TRUSTEE_DID)), rich_schema.clone());
        trustee.sign_request(&mut rs_req_1);
        test_pool.send_request(&rs_req_1).unwrap();

        // Send the same RichSchema object.
        // Expected behaviour for now:
        // Reject with reason 'The action is forbidden',
        // because we do not allow to edit already existed RichSchema

        let mut rs_req_2 =
            builder::_build_rs_req(DidValue(String::from(TRUSTEE_DID)), rich_schema.clone());
        trustee.sign_request(&mut rs_req_2);

        let err = test_pool.send_request(&rs_req_2).unwrap_err();
        helpers::check_response_type(&err, "REJECT");
        helpers::match_response_error(&err, "The action is forbidden");
    }

    #[rstest]
    fn test_rs_already_exist_with_the_same_metadata(trustee: Identity, test_pool: TestPool) {
        let rich_schema = builder::_rich_schema();
        let mut rs_req_1 =
            builder::_build_rs_req(DidValue(String::from(TRUSTEE_DID)), rich_schema.clone());
        trustee.sign_request(&mut rs_req_1);
        test_pool.send_request(&rs_req_1).unwrap();

        // Send another RichSchema request with the same rsName, rsType and rsVersion
        // but with different id and content.
        // Expected behaviour:
        // Reject with error that 'already exists'

        let new_id = builder::_rs_id();
        let rich_schema_2 = RichSchema::new(
            new_id.clone(),
            builder::_rs_content(new_id),
            rich_schema.rs_name,
            rich_schema.rs_version,
            rich_schema.rs_type,
            rich_schema.ver,
        );
        let mut rs_req_2 =
            builder::_build_rs_req(DidValue(String::from(TRUSTEE_DID)), rich_schema_2);
        trustee.sign_request(&mut rs_req_2);

        let err = test_pool.send_request(&rs_req_2).unwrap_err();
        helpers::check_response_type(&err, "REJECT");
        helpers::match_response_error(&err, "already exists");
    }

    #[rstest(
        rs_content,
        case(builder::_rs_content(builder::_rs_id())),
        case(RSContent(r#"{"@id": "there:is:id:field", "some": "other", "valid": "fields"}"#.to_string())),
        case(RSContent(r#"{"@type": "sch", "some": "other", "valid": "fields"}"#.to_string())),
        case(RSContent("{not: valid; json: string}".to_string()))
    )]
    fn test_rs_request_wrong_json(rs_content: RSContent, trustee: Identity, test_pool: TestPool) {
        let mut rich_schema = builder::_rich_schema();

        // Put a JSON-LD string with different id in content and at the top of data
        rich_schema.content = rs_content;

        let mut rs_req = builder::_build_rs_req(DidValue(String::from(TRUSTEE_DID)), rich_schema);
        trustee.sign_request(&mut rs_req);

        let err = test_pool.send_request(&rs_req).unwrap_err();
        helpers::check_response_type(&err, "REQNACK");
    }
}
