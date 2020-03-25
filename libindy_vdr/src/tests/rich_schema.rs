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
    use rstest::*;

    fn rs_id_str() -> String {
        let mut id = "did:sov:".to_string();
        id.push_str(&get_rand_string(32));
        return id;
    }

    pub fn rs_id() -> RichSchemaId {
        return RichSchemaId::new(rs_id_str());
    }

    pub fn rs_version() -> String {
        let mut rng = rand::thread_rng();
        return format!(
            "{}.{}.{}",
            rng.gen::<u32>(),
            rng.gen::<u32>(),
            rng.gen::<u32>()
        )
        .to_string();
    }

    pub fn rs_content(rs_id: RichSchemaId) -> RSContent {
        let rs_as_json = json!({
            "@id": rs_id,
            "@type": constants::RS_SCHEMA_TYPE_VALUE.to_string(),
            "some": "other".to_string(),
            "valid": "objects".to_string(),
        });
        return RSContent(rs_as_json.to_string());
    }

    pub fn rich_schema() -> RichSchema {
        let rs_id = rs_id();
        RichSchema::new(
            rs_id.clone(),
            rs_content(rs_id),
            "test_rich_schema".to_string(),
            rs_version(),
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
    fn test_rs_with_wrong_rs_type() {
        let mut rich_schema = rich_schema();
        rich_schema.rs_type = "SomeOtherType".to_string();
        let pool = TestPool::new();
        let err = pool
            .request_builder()
            .build_rich_schema_request(
                &DidValue(String::from(TRUSTEE_DID)),
                rich_schema.id,
                rich_schema.content,
                rich_schema.rs_name,
                rich_schema.rs_version,
                rich_schema.rs_type,
                rich_schema.ver,
            )
            .unwrap_err();
        assert!(err.to_string().contains("unknown variant `SomeOtherType`"));
    }

    #[test]
    fn test_rs_request_general() {
        let rich_schema = rich_schema();
        let rs_req = _build_rs_req(DidValue(String::from(TRUSTEE_DID)), rich_schema.clone());
        let expected_result = json!({
            "type": constants::RICH_SCHEMA,
            "id": rich_schema.id,
            "content": rs_content(rich_schema.id),
            "rsName": "test_rich_schema".to_string(),
            "rsVersion": rich_schema.rs_version,
            "rsType": constants::RS_SCHEMA_TYPE_VALUE.to_string(),
            "ver": "1".to_string()
        });
        helpers::check_request_operation(&rs_req, expected_result);
    }

    #[rstest(
    rs_type => [
    "sch".to_string(),
    "ctx".to_string(),
    "map".to_string(),
    "enc".to_string(),
    "cdf".to_string(),
    "pdf".to_string(),
    ]
    )]
    fn test_rs_request_works_for_fully_qualified_did(rs_type: String) {
        let mut rich_schema = rich_schema();
        rich_schema.rs_type = rs_type.clone();
        let rs_req = _build_rs_req(DidValue(String::from(TRUSTEE_DID_FQ)), rich_schema.clone());
        let expected_result = json!({
            "type": constants::RS_TYPE_TO_OP.get(&rich_schema.rs_type.as_str()).unwrap().to_string(),
            "id": rich_schema.id,
            "content": rs_content(rich_schema.id),
            "rsName": "test_rich_schema".to_string(),
            "rsVersion": rich_schema.rs_version,
            "rsType": rs_type,
            "ver": "1".to_string()
        });
        helpers::check_request_operation(&rs_req, expected_result);
    }
}

mod sender {
    use super::*;
    use crate::ledger::requests::rich_schema::RSContent;
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
        let rich_schema = builder::rich_schema();
        let rs_id = rich_schema.clone();
        let rs_meta = rich_schema.clone();
        let mut rs_req =
            builder::_build_rs_req(DidValue(String::from(TRUSTEE_DID)), rich_schema.clone());
        trustee.sign_request(&mut rs_req);

        let rs_response = test_pool.send_request(&rs_req).unwrap();
        let expected_result = json!({
            "id": rich_schema.id,
            "content": builder::rs_content(rich_schema.id),
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

    #[rstest(
        rs_type => [
        "sch".to_string(),
        "ctx".to_string(),
        "map".to_string(),
        "enc".to_string(),
        "cdf".to_string(),
        "pdf".to_string(),
        ],
        rs_content => [
        builder::rs_content(builder::rs_id()),
        RSContent(r#"{"@id": "there:is:id:field", "some": "other", "valid": "fields"}"#.to_string()),
        RSContent(r#"{"@type": "sch", "some": "other", "valid": "fields"}"#.to_string()),
        RSContent("{not: valid; json: string}".to_string()),
        ],
    )]
    fn test_rs_request_wrong_json(
        rs_content: RSContent,
        trustee: Identity,
        test_pool: TestPool,
        rs_type: String,
    ) {
        let mut rich_schema = builder::rich_schema();
        rich_schema.rs_type = rs_type;

        // Put a JSON-LD string with different id in content and at the top of data
        rich_schema.content = rs_content;

        let mut rs_req = builder::_build_rs_req(DidValue(String::from(TRUSTEE_DID)), rich_schema);
        trustee.sign_request(&mut rs_req);

        let err = test_pool.send_request(&rs_req).unwrap_err();
        helpers::check_response_type(&err, "REQNACK");
    }
}

mod rs_chain {
    use super::*;
    use crate::ledger::constants::{
        RS_CONTEXT_TYPE_VALUE, RS_CRED_DEF_TYPE_VALUE, RS_ENCODING_TYPE_VALUE,
        RS_MAPPING_TYPE_VALUE, RS_PRES_DEF_TYPE_VALUE, RS_SCHEMA_TYPE_VALUE,
    };
    use crate::ledger::requests::rich_schema::{RSContent, RichSchema};
    use crate::ledger::PreparedRequest;
    use crate::tests::utils::crypto::Identity;
    use builder;

    pub struct RSChain {
        pub rs_sch_id: RichSchemaId,
        pub rs_map_id: RichSchemaId,
        pub rs_ctx_id: RichSchemaId,
        pub rs_enc_id: RichSchemaId,
        pub rs_cdf_id: RichSchemaId,
        pub rs_pdf_id: RichSchemaId,
    }

    impl RSChain {
        pub fn new() -> RSChain {
            let rs_sch_id = builder::rs_id();
            let rs_map_id = builder::rs_id();
            let rs_ctx_id = builder::rs_id();
            let rs_enc_id = builder::rs_id();
            let rs_cdf_id = builder::rs_id();
            let rs_pdf_id = builder::rs_id();
            RSChain {
                rs_sch_id,
                rs_map_id,
                rs_ctx_id,
                rs_enc_id,
                rs_cdf_id,
                rs_pdf_id,
            }
        }

        pub fn make_rs_sch(&self) -> RichSchema {
            RichSchema::new(
                self.rs_sch_id.clone(),
                self._make_rs_sch_content(),
                "test_rich_schema".to_string(),
                builder::rs_version(),
                RS_SCHEMA_TYPE_VALUE.to_string(),
                "1.0.0".to_string(),
            )
        }
        fn _make_rs_sch_content(&self) -> RSContent {
            let json_c = json!({
                "@id": self.rs_sch_id.clone(),
                "@context": self.rs_ctx_id,
                "@type": "rdfs:Class".to_string(),
                "driver": "Driver",
                "dateOfIssue": "Date",
                "dateOfExpiry": "Date",
                "issuingAuthority": "Text",
                "licenseNumber": "Text",
                "categoriesOfVehicles": {
                    "vehicleType": "Text",
                    "vehicleType-input": {
                        "@type": "sch:PropertyValueSpecification",
                        "valuePattern": "^(A|B|C|D|BE|CE|DE|AM|A1|A2|B1|C1|D1|C1E|D1E)$"
                    },
                    "dateOfIssue": "Date",
                    "dateOfExpiry": "Date",
                    "restrictions": "Text",
                    "restrictions-input": {
                        "@type": "sch:PropertyValueSpecification",
                        "valuePattern": "^([A-Z]|[1-9])$"
                    }
                },
                "administrativeNumber": "Text"
            });
            return RSContent(json_c.to_string());
        }

        pub fn make_rs_ctx(&self) -> RichSchema {
            RichSchema::new(
                self.rs_ctx_id.clone(),
                self._make_rs_ctx_content(),
                "test_rich_schema_context".to_string(),
                builder::rs_version(),
                RS_CONTEXT_TYPE_VALUE.to_string(),
                "1.0.0".to_string(),
            )
        }

        fn _make_rs_ctx_content(&self) -> RSContent {
            let json_c = json!({
            "@context": [
                {
                    "@version": "1.1",
                },
                "https://www.w3.org/ns/odrl.jsonld",
                {
                    "ex": "https://example.org/examples#",
                    "schema": "http://schema.org/",
                    "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#"
                }
            ]
            });
            return RSContent(json_c.to_string());
        }

        pub fn make_rs_map(&self) -> RichSchema {
            RichSchema::new(
                self.rs_map_id.clone(),
                self._make_rs_map_content(),
                "test_rich_schema_map".to_string(),
                builder::rs_version(),
                RS_MAPPING_TYPE_VALUE.to_string(),
                "1.0.0".to_string(),
            )
        }

        fn _make_rs_map_content(&self) -> RSContent {
            let json_c = json!({
                "@id": self.rs_map_id.clone(),
                "@context": self.rs_ctx_id.clone(),
                "@type": "rdfs:Class",
                "schema": self.rs_sch_id.clone(),
                "attributes": {
                    "driver": [{
                        "enc": self.rs_enc_id.clone(),
                        "rank": 5
                    }],
                    "dateOfIssue": [{
                        "enc": self.rs_enc_id.clone(),
                        "rank": 4
                    }],
                    "issuingAuthority": [{
                        "enc": self.rs_enc_id.clone(),
                        "rank": 3
                    }],
                    "licenseNumber": [
                        {
                            "enc": self.rs_enc_id.clone(),
                            "rank": 1
                        },
                        {
                            "enc": self.rs_enc_id.clone(),
                            "rank": 2
                        },
                    ],
                    "categoriesOfVehicles": {
                    "vehicleType": [{
                        "enc": self.rs_enc_id.clone(),
                        "rank": 6
                    }],
                    "dateOfIssue": [{
                        "enc": self.rs_enc_id.clone(),
                        "rank": 7
                    }],
                    },
                    "administrativeNumber": [{
                        "enc": self.rs_enc_id.clone(),
                        "rank": 8
                    }]
                }
            });
            return RSContent(json_c.to_string());
        }

        pub fn make_rs_enc(&self) -> RichSchema {
            RichSchema::new(
                self.rs_enc_id.clone(),
                self._make_rs_enc_content(),
                "test_rich_schema_enc".to_string(),
                builder::rs_version(),
                RS_ENCODING_TYPE_VALUE.to_string(),
                "1.0.0".to_string(),
            )
        }

        fn _make_rs_enc_content(&self) -> RSContent {
            let json_c = json!({
                "input": {
                "id": "DateRFC3339",
                "type": "string"
                },
                "output": {
                    "id": "UnixTime",
                    "type": "256-bit integer"
                },
                "algorithm": {
                    "description": "This encoding transforms an RFC3339 - formatted datetime object into the number of seconds since January 1, 1970(the Unix epoch).",
                    "documentation": "https://github.com/hyperledger/indy-hipe/commit/3a39665fd384254f08316eef6230c2f411b8f765",
                    "implementation": "https://github.com/hyperledger/indy-hipe/commit/3a39665fd384254f08316eef6230c2f411b8f869",
                },
                "testVectors": "https://github.com/hyperledger/indy-hipe/commit/3a39665fd384254f08316eef6230c2f411b8f766"
            });
            return RSContent(json_c.to_string());
        }

        pub fn make_rs_cdf(&self) -> RichSchema {
            RichSchema::new(
                self.rs_cdf_id.clone(),
                self._make_rs_cdf_content(),
                "test_rich_schema_cdf".to_string(),
                builder::rs_version(),
                RS_CRED_DEF_TYPE_VALUE.to_string(),
                "1.0.0".to_string(),
            )
        }

        fn _make_rs_cdf_content(&self) -> RSContent {
            let json_c = json!({
                "signatureType": "CL",
                "mapping": self.rs_map_id.clone(),
                "schema": self.rs_sch_id.clone(),
                "publicKey": {
                    "primary": "...",
                    "revocation": "...",
                }
            });
            return RSContent(json_c.to_string());
        }

        pub fn make_rs_pdf(&self) -> RichSchema {
            RichSchema::new(
                self.rs_pdf_id.clone(),
                self._make_rs_pdf_content(),
                "test_rich_schema_pdf".to_string(),
                builder::rs_version(),
                RS_PRES_DEF_TYPE_VALUE.to_string(),
                "1.0.0".to_string(),
            )
        }

        fn _make_rs_pdf_content(&self) -> RSContent {
            let json_c = json!({
                "@id": self.rs_pdf_id.clone(),
                "@context": self.rs_ctx_id.clone(),
                "@type": "rdfs:Class",
                "attr1": "",
                "attr2": ""
            });
            return RSContent(json_c.to_string());
        }
    }

    fn send_rs_obj(rs_obj: RichSchema) -> Result<String, String> {
        let pool = TestPool::new();
        let rs_req = make_signed_req_from_rs_obj(rs_obj);
        Ok(pool.send_request(&rs_req).unwrap())
    }

    fn make_signed_req_from_rs_obj(rs_obj: RichSchema) -> PreparedRequest {
        let pool = TestPool::new();
        let trustee = Identity::trustee();
        let mut rs_req = pool
            .request_builder()
            .build_rich_schema_request(
                &DidValue(String::from(TRUSTEE_DID)),
                rs_obj.id,
                rs_obj.content,
                rs_obj.rs_name,
                rs_obj.rs_version,
                rs_obj.rs_type,
                rs_obj.ver,
            )
            .unwrap();
        trustee.sign_request(&mut rs_req);
        return rs_req;
    }

    fn make_get_req_by_id_from_rs_obj(rs_obj: RichSchema) -> PreparedRequest {
        let pool = TestPool::new();
        pool.request_builder()
            .build_get_rich_schema_by_id(&DidValue(String::from(TRUSTEE_DID)), &rs_obj.id)
            .unwrap()
    }

    fn make_get_req_by_metadata_from_rs_obj(rs_obj: RichSchema) -> PreparedRequest {
        let pool = TestPool::new();
        pool.request_builder()
            .build_get_rich_schema_by_metadata(
                &DidValue(String::from(TRUSTEE_DID)),
                rs_obj.rs_type,
                rs_obj.rs_name,
                rs_obj.rs_version,
            )
            .unwrap()
    }

    #[test]
    fn test_general_rs_chain() {
        let pool = TestPool::new();
        let rs_chain = RSChain::new();
        let trustee = Identity::trustee();
        let rs_objects = vec![
            rs_chain.make_rs_sch(),
            rs_chain.make_rs_ctx(),
            rs_chain.make_rs_enc(),
            rs_chain.make_rs_map(),
            rs_chain.make_rs_cdf(),
            rs_chain.make_rs_pdf(),
        ];
        // Write all of the RichSchema objects to ledger
        for rs_obj in rs_objects.clone() {
            send_rs_obj(rs_obj);
        }
        // Check, that all of objects are written to ledger
        for rs_obj in rs_objects.clone() {
            let mut get_req_req_by_ib = make_get_req_by_id_from_rs_obj(rs_obj.clone());
            let mut get_req_req_by_meta = make_get_req_by_metadata_from_rs_obj(rs_obj);
            trustee.sign_request(&mut get_req_req_by_ib);
            trustee.sign_request(&mut get_req_req_by_meta);
            pool.send_request(&get_req_req_by_ib).unwrap();
            pool.send_request(&get_req_req_by_meta).unwrap();
        }
    }
}
