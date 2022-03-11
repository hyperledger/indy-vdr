use crate::utils::Qualifiable;
use chrono::{DateTime, Utc};
use futures_executor::block_on;
use serde_json::Value as SJsonValue;

use super::did::{DidUrl, LedgerObject, QueryParameter};
use super::did_document::{DidDocument, LEGACY_INDY_SERVICE};
use crate::common::error::prelude::*;
use crate::ledger::responses::{Endpoint, GetNymResultV1};

use crate::ledger::identifiers::{CredentialDefinitionId, RevocationRegistryId, SchemaId};
use crate::ledger::{constants, RequestBuilder};
use crate::pool::helpers::perform_ledger_request;
use crate::pool::{Pool, PoolRunner, PreparedRequest, RequestResult, TimingResult};
use crate::utils::did::DidValue;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Result {
    DidDocument(DidDocument),
    Content(SJsonValue),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentMetadata {
    node_response: SJsonValue,
    object_type: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionResult {
    did_resolution_metadata: Option<String>,
    did_document: Option<SJsonValue>,
    did_document_metadata: Option<ContentMetadata>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DereferencingResult {
    dereferencing_metadata: Option<String>,
    content_stream: Option<SJsonValue>,
    content_metadata: Option<ContentMetadata>,
}

// DID (URL) Resolver for did:indy method
pub struct PoolResolver<T: Pool> {
    pool: T,
}

impl<T: Pool> PoolResolver<T> {
    pub fn new(pool: T) -> PoolResolver<T> {
        PoolResolver { pool }
    }

    pub fn dereference(&self, did_url: &str) -> VdrResult<String> {
        let (data, metadata) = self._resolve(did_url)?;

        let content = match data {
            Result::Content(c) => Some(c),
            _ => None,
        };

        let result = DereferencingResult {
            dereferencing_metadata: None,
            content_stream: content,
            content_metadata: Some(metadata),
        };

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    pub fn resolve(&self, did: &str) -> VdrResult<String> {
        let (data, metadata) = self._resolve(did)?;

        let diddoc = match data {
            Result::DidDocument(doc) => Some(doc.to_value()?),
            _ => None,
        };
        let result = ResolutionResult {
            did_resolution_metadata: None,
            did_document: diddoc,
            did_document_metadata: Some(metadata),
        };

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    pub fn _resolve(&self, did: &str) -> VdrResult<(Result, ContentMetadata)> {
        let did_url = DidUrl::from_str(did)?;

        let builder = self.pool.get_request_builder();
        let request = build_request(&did_url, &builder)?;

        let ledger_data = handle_request(&self.pool, &request)?;
        let data = parse_ledger_data(&ledger_data)?;

        let (result, object_type) = match request.txn_type.as_str() {
            constants::GET_NYM => {
                let get_nym_result: GetNymResultV1 =
                    serde_json::from_str(data.as_str().unwrap())
                        .map_err(|_| err_msg(VdrErrorKind::Resolver, "Could not parse NYM data"))?;

                let endpoint: Option<Endpoint> = if get_nym_result.diddoc_content.is_none() {
                    // Legacy: Try to find an attached ATTRIBUTE transacation with raw endpoint
                    self.fetch_legacy_endpoint(&did_url.id).ok()
                } else {
                    None
                };

                let did_document = DidDocument::new(
                    &did_url.namespace,
                    &get_nym_result.dest,
                    &get_nym_result.verkey,
                    endpoint,
                    None,
                );
                (Result::DidDocument(did_document), String::from("NYM"))
            }
            constants::GET_CRED_DEF => (Result::Content(data), String::from("CRED_DEF")),
            constants::GET_SCHEMA => (Result::Content(data), String::from("SCHEMA")),
            constants::GET_REVOC_REG_DEF => (Result::Content(data), String::from("REVOC_REG_DEF")),
            constants::GET_REVOC_REG_DELTA => {
                (Result::Content(data), String::from("REVOC_REG_DELTA"))
            }
            _ => (Result::Content(data), String::from("UNKOWN")),
        };

        let metadata = ContentMetadata {
            node_response: serde_json::from_str(&ledger_data).unwrap(),
            object_type,
        };

        let result_with_metadata = (result, metadata);

        Ok(result_with_metadata)
    }

    fn fetch_legacy_endpoint(&self, did: &DidValue) -> VdrResult<Endpoint> {
        let builder = self.pool.get_request_builder();
        let request = builder.build_get_attrib_request(
            None,
            did,
            Some(String::from(LEGACY_INDY_SERVICE)),
            None,
            None,
        )?;
        let ledger_data = handle_request(&self.pool, &request)?;
        let endpoint_data = parse_ledger_data(&ledger_data)?;
        let endpoint_data: Endpoint = serde_json::from_str(endpoint_data.as_str().unwrap())
            .map_err(|_| err_msg(VdrErrorKind::Resolver, "Could not parse endpoint data"))?;
        Ok(endpoint_data)
    }
}

fn build_request(did: &DidUrl, builder: &RequestBuilder) -> VdrResult<PreparedRequest> {
    let request = if did.path.is_some() {
        match LedgerObject::from_str(did.path.as_ref().unwrap().as_str())? {
            LedgerObject::Schema(schema) => builder.build_get_schema_request(
                None,
                &SchemaId::new(&did.id, &schema.name, &schema.version),
            ),
            LedgerObject::ClaimDef(claim_def) => builder.build_get_cred_def_request(
                None,
                &CredentialDefinitionId::from_str(
                    format!(
                        "{}:3:CL:{}:{}",
                        &did.id, claim_def.schema_seq_no, claim_def.name
                    )
                    .as_str(),
                )
                .unwrap(),
            ),
            LedgerObject::RevRegDef(rev_reg_def) => builder.build_get_revoc_reg_def_request(
                None,
                &RevocationRegistryId::from_str(
                    format!(
                        "{}:4:{}:3:CL:{}:{}:CL_ACCUM:{}",
                        &did.id,
                        &did.id,
                        rev_reg_def.schema_seq_no,
                        rev_reg_def.claim_def_name,
                        rev_reg_def.tag
                    )
                    .as_str(),
                )
                .unwrap(),
            ),
            LedgerObject::RevRegEntry(rev_reg_entry) => {
                // If From or To parameters, return RevRegDelta request
                if did.query.contains_key(&QueryParameter::From)
                    || did.query.contains_key(&QueryParameter::To)
                {
                    let mut from: Option<i64> = None;
                    if did.query.contains_key(&QueryParameter::From) {
                        from = did
                            .query
                            .get(&QueryParameter::From)
                            .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
                            .and_then(|d| Some(d.timestamp()));
                    }

                    let to = parse_or_now(did.query.get(&QueryParameter::To))?;

                    builder.build_get_revoc_reg_delta_request(
                        None,
                        &RevocationRegistryId::from_str(
                            format!(
                                "{}:4:{}:3:CL:{}:{}:CL_ACCUM:{}",
                                &did.id,
                                &did.id,
                                rev_reg_entry.schema_seq_no,
                                rev_reg_entry.claim_def_name,
                                rev_reg_entry.tag
                            )
                            .as_str(),
                        )
                        .unwrap(),
                        from,
                        to,
                    )
                // Else return RevRegEntry request
                } else {
                    let timestamp = parse_or_now(did.query.get(&QueryParameter::VersionTime))?;

                    builder.build_get_revoc_reg_request(
                        None,
                        &RevocationRegistryId::from_str(
                            format!(
                                "{}:4:{}:3:CL:{}:{}:CL_ACCUM:{}",
                                &did.id,
                                &did.id,
                                rev_reg_entry.schema_seq_no,
                                rev_reg_entry.claim_def_name,
                                rev_reg_entry.tag
                            )
                            .as_str(),
                        )
                        .unwrap(),
                        timestamp,
                    )
                }
            }
            // This path is deprecated. Deltas can be retrieved through RevRegEntry
            LedgerObject::RevRegDelta(rev_reg_delta) => {
                let mut from: Option<i64> = None;
                if did.query.contains_key(&QueryParameter::From) {
                    from = did
                        .query
                        .get(&QueryParameter::From)
                        .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
                        .and_then(|d| Some(d.timestamp()));
                }

                let to = parse_or_now(did.query.get(&QueryParameter::To))?;

                builder.build_get_revoc_reg_delta_request(
                    None,
                    &RevocationRegistryId::from_str(
                        format!(
                            "{}:4:{}:3:CL:{}:{}:CL_ACCUM:{}",
                            &did.id,
                            &did.id,
                            rev_reg_delta.schema_seq_no,
                            rev_reg_delta.claim_def_name,
                            rev_reg_delta.tag
                        )
                        .as_str(),
                    )
                    .unwrap(),
                    from,
                    to,
                )
            }
        }
    } else {
        let seq_no: Option<i32> = did
            .query
            .get(&QueryParameter::VersionId)
            .and_then(|v| v.parse().ok());
        let timestamp: Option<u64> = did
            .query
            .get(&QueryParameter::VersionTime)
            .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
            .and_then(|d| Some(d.timestamp()))
            .and_then(|d| Some(d as u64));

        builder.build_get_nym_request(Option::None, &did.id, seq_no, timestamp)
    };
    request
}

fn handle_request<T: Pool>(pool: &T, request: &PreparedRequest) -> VdrResult<String> {
    let (result, _timing) = block_on(request_transaction(pool, &request))?;
    match result {
        RequestResult::Reply(data) => Ok(data),
        RequestResult::Failed(error) => Err(error),
    }
}

async fn request_transaction<T: Pool>(
    pool: &T,
    request: &PreparedRequest,
) -> VdrResult<(RequestResult<String>, Option<TimingResult>)> {
    perform_ledger_request(pool, &request).await
}

pub struct PoolRunnerResolver<'a> {
    runner: &'a PoolRunner,
}

impl<'a> PoolRunnerResolver<'a> {
    pub fn new(runner: &'a PoolRunner) -> PoolRunnerResolver {
        PoolRunnerResolver { runner }
    }

    // pub fn dereference(&self, did_url: &str, callback: Callback<SendReqResponse>) -> VdrResult<String> {
    //     let (data, metadata) = self._resolve(did_url)?;

    //     let content = match data {
    //         Result::Content(c) => Some(c),
    //         _ => None,
    //     };

    //     let result = DereferencingResult {
    //         dereferencing_metadata: None,
    //         content_stream: content,
    //         content_metadata: Some(metadata),
    //     };

    //     Ok(serde_json::to_string_pretty(&result).unwrap())
    // }

    pub fn resolve(&self, did: &str, callback: Callback<VdrResult<String>>) -> VdrResult<()> {
        self._resolve(
            did,
            Box::new(|result| {
                let (data, metadata) = result.unwrap();
                let diddoc = match data {
                    Result::DidDocument(doc) => Some(doc.to_value().unwrap()),
                    _ => None,
                };
                let result = ResolutionResult {
                    did_resolution_metadata: None,
                    did_document: diddoc,
                    did_document_metadata: Some(metadata),
                };

                callback(Ok(serde_json::to_string_pretty(&result).unwrap()))
            }),
        )
    }

    fn _resolve(
        &self,
        did: &str,
        callback: Callback<VdrResult<(Result, ContentMetadata)>>,
    ) -> VdrResult<()> {
        let did_url = DidUrl::from_str(did)?;

        let builder = RequestBuilder::default();
        let request = build_request(&did_url, &builder)?;
        let txn_type = request.txn_type.clone();
        self.runner.send_request(
            request,
            Box::new(
                move |result: VdrResult<(RequestResult<String>, Option<TimingResult>)>| {
                    match result {
                        Ok((res, _)) => {
                            match res {
                                RequestResult::Reply(reply_data) => {
                                    let data = parse_ledger_data(&reply_data).unwrap();
                                    let (result, object_type) = match txn_type.as_str() {
                                        constants::GET_NYM => {
                                            let get_nym_result: GetNymResultV1 =
                                                serde_json::from_str(data.as_str().unwrap())
                                                    .unwrap();

                                            let endpoint: Option<Endpoint> =
                                                if get_nym_result.diddoc_content.is_none() {
                                                    // Legacy: Try to find an attached ATTRIBUTE transacation with raw endpoint
                                                    // self.fetch_legacy_endpoint(&did_url.id).ok()
                                                    None
                                                } else {
                                                    None
                                                };

                                            let did_document = DidDocument::new(
                                                &did_url.namespace,
                                                &get_nym_result.dest,
                                                &get_nym_result.verkey,
                                                endpoint,
                                                None,
                                            );
                                            (Result::DidDocument(did_document), String::from("NYM"))
                                        }
                                        constants::GET_CRED_DEF => {
                                            (Result::Content(data), String::from("CRED_DEF"))
                                        }
                                        constants::GET_SCHEMA => {
                                            (Result::Content(data), String::from("SCHEMA"))
                                        }
                                        constants::GET_REVOC_REG_DEF => {
                                            (Result::Content(data), String::from("REVOC_REG_DEF"))
                                        }
                                        constants::GET_REVOC_REG_DELTA => {
                                            (Result::Content(data), String::from("REVOC_REG_DELTA"))
                                        }
                                        _ => (Result::Content(data), String::from("UNKOWN")),
                                    };

                                    let metadata = ContentMetadata {
                                        node_response: serde_json::from_str(&reply_data).unwrap(),
                                        object_type,
                                    };

                                    let result_with_metadata = (result, metadata);
                                    callback(Ok(result_with_metadata));
                                }
                                RequestResult::Failed(err) => callback(Err(err)),
                            }
                        }
                        Err(err) => callback(Err(err)),
                    }
                },
            ),
        )
    }
}

type Callback<R> = Box<dyn (FnOnce(R) -> ()) + Send>;
// type SendReqResponse = VdrResult<(RequestResult<String>, Option<TimingResult>)>;

fn parse_ledger_data(ledger_data: &str) -> VdrResult<SJsonValue> {
    let v: SJsonValue = serde_json::from_str(&ledger_data)
        .map_err(|_| err_msg(VdrErrorKind::Resolver, "Could not parse ledger response"))?;
    let data: &SJsonValue = &v["result"]["data"];
    if *data == SJsonValue::Null {
        Err(err_msg(
            VdrErrorKind::Resolver,
            format!("Empty data in ledger response"),
        ))
    } else {
        Ok(data.to_owned())
    }
}

fn parse_or_now(datetime: Option<&String>) -> VdrResult<i64> {
    match datetime {
        Some(datetime) => {
            let dt = DateTime::parse_from_rfc3339(datetime).map_err(|_| {
                err_msg(
                    VdrErrorKind::Resolver,
                    format!("Could not parse datetime {}", datetime),
                )
            })?;
            Ok(dt.timestamp())
        }
        None => Ok(Utc::now().timestamp()),
    }
}

#[cfg(test)]
mod tests {

    use urlencoding::encode;

    use super::*;
    use rstest::*;

    use crate::pool::ProtocolVersion;

    #[fixture]
    fn request_builder() -> RequestBuilder {
        RequestBuilder::new(ProtocolVersion::Node1_4)
    }

    #[rstest]
    fn build_get_revoc_reg_request_from_version_time(request_builder: RequestBuilder) {
        let datetime_as_str = "2020-12-20T19:17:47Z";
        let did_url_as_str = format!("did:indy:idunion:Dk1fRRTtNazyMuK2cr64wp/anoncreds/v0/REV_REG_ENTRY/104/revocable/a4e25e54?versionTime={}",datetime_as_str);
        let did_url = DidUrl::from_str(&did_url_as_str).unwrap();
        let request = build_request(&did_url, &request_builder).unwrap();
        let timestamp = (*(request.req_json).get("operation").unwrap())
            .get("timestamp")
            .unwrap()
            .as_u64()
            .unwrap() as i64;
        assert_eq!(constants::GET_REVOC_REG, request.txn_type);

        assert_eq!(
            DateTime::parse_from_rfc3339(datetime_as_str)
                .unwrap()
                .timestamp(),
            timestamp
        );
    }

    #[rstest]
    fn build_get_revoc_reg_without_version_time(request_builder: RequestBuilder) {
        let now = chrono::Utc::now().timestamp();

        let did_url_as_str = "did:indy:idunion:Dk1fRRTtNazyMuK2cr64wp/anoncreds/v0/REV_REG_ENTRY/104/revocable/a4e25e54";
        let did_url = DidUrl::from_str(did_url_as_str).unwrap();
        let request = build_request(&did_url, &request_builder).unwrap();
        let timestamp = (*(request.req_json).get("operation").unwrap())
            .get("timestamp")
            .unwrap()
            .as_u64()
            .unwrap() as i64;

        assert_eq!(constants::GET_REVOC_REG, request.txn_type);
        assert!(timestamp >= now);
    }

    #[rstest]
    fn build_get_revoc_reg_request_fails_with_unparsable_version_time(
        request_builder: RequestBuilder,
    ) {
        let datetime_as_str = "20201220T19:17:47Z";
        let did_url_as_str = format!("did:indy:idunion:Dk1fRRTtNazyMuK2cr64wp/anoncreds/v0/REV_REG_ENTRY/104/revocable/a4e25e54?versionTime={}",datetime_as_str);
        let did_url = DidUrl::from_str(&did_url_as_str).unwrap();
        let _err = build_request(&did_url, &request_builder).unwrap_err();
    }

    #[rstest]
    fn build_get_revoc_reg_delta_request_with_from_to(request_builder: RequestBuilder) {
        let from_as_str = "2019-12-20T19:17:47Z";
        let to_as_str = "2020-12-20T19:17:47Z";
        let did_url_as_str = format!("did:indy:idunion:Dk1fRRTtNazyMuK2cr64wp/anoncreds/v0/REV_REG_ENTRY/104/revocable/a4e25e54?from={}&to={}",from_as_str, to_as_str);
        let did_url = DidUrl::from_str(&did_url_as_str).unwrap();
        let request = build_request(&did_url, &request_builder).unwrap();
        assert_eq!(request.txn_type, constants::GET_REVOC_REG_DELTA);
    }

    #[rstest]
    fn build_get_revoc_reg_delta_request_with_from_only(request_builder: RequestBuilder) {
        let now = chrono::Utc::now().timestamp();
        let from_as_str = "2019-12-20T19:17:47Z";
        let did_url_as_str = format!("did:indy:idunion:Dk1fRRTtNazyMuK2cr64wp/anoncreds/v0/REV_REG_ENTRY/104/revocable/a4e25e54?from={}",from_as_str);
        let did_url = DidUrl::from_str(&did_url_as_str).unwrap();
        let request = build_request(&did_url, &request_builder).unwrap();

        let to = (*(request.req_json).get("operation").unwrap())
            .get("to")
            .unwrap()
            .as_u64()
            .unwrap() as i64;
        assert_eq!(request.txn_type, constants::GET_REVOC_REG_DELTA);
        assert!(to >= now)
    }

    #[rstest]
    fn build_get_revoc_reg_delta_request_without_parameter(request_builder: RequestBuilder) {
        let now = chrono::Utc::now().timestamp();
        let did_url_as_str = "did:indy:idunion:Dk1fRRTtNazyMuK2cr64wp/anoncreds/v0/REV_REG_DELTA/104/revocable/a4e25e54";
        let did_url = DidUrl::from_str(did_url_as_str).unwrap();
        let request = build_request(&did_url, &request_builder).unwrap();

        let to = (*(request.req_json).get("operation").unwrap())
            .get("to")
            .unwrap()
            .as_u64()
            .unwrap() as i64;

        let from = (*(request.req_json).get("operation").unwrap()).get("from");
        assert_eq!(request.txn_type, constants::GET_REVOC_REG_DELTA);
        assert!(from.is_none());
        assert!(to >= now);
    }

    #[rstest]
    fn build_get_schema_request_with_whitespace(request_builder: RequestBuilder) {
        let name = "My Schema";
        let encoded_schema_name = encode(name).to_string();
        let did_url_string = format!(
            "did:indy:idunion:Dk1fRRTtNazyMuK2cr64wp/anoncreds/v0/SCHEMA/{}/1.0",
            encoded_schema_name
        );

        let did_url = DidUrl::from_str(did_url_string.as_str()).unwrap();
        let request = build_request(&did_url, &request_builder).unwrap();
        let schema_name = (*(request.req_json).get("operation").unwrap())
            .get("data")
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .unwrap();
        assert_eq!(schema_name, name);
    }
}
