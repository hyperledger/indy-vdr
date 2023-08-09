use serde_json::Value as SJsonValue;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use super::did::{DidUrl, LedgerObject, QueryParameter};
use super::did_document::{DidDocument, LEGACY_INDY_SERVICE};
use super::types::*;

use crate::common::error::prelude::*;
use crate::ledger::constants;
use crate::ledger::identifiers::{CredentialDefinitionId, RevocationRegistryId, SchemaId};
use crate::ledger::responses::{Endpoint, GetNymResultV1};
use crate::ledger::RequestBuilder;
use crate::pool::helpers::perform_ledger_request;
use crate::pool::{Pool, PreparedRequest, RequestResult, TimingResult};
use crate::utils::did::DidValue;
use crate::utils::Qualifiable;

pub fn build_request(did: &DidUrl, builder: &RequestBuilder) -> VdrResult<PreparedRequest> {
    let request = if did.path.is_some() {
        match LedgerObject::parse(did.path.as_ref().unwrap().as_str())? {
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
                            .and_then(|d| OffsetDateTime::parse(d, &Rfc3339).ok())
                            .map(|d| d.unix_timestamp());
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
                        .and_then(|d| OffsetDateTime::parse(d, &Rfc3339).ok())
                        .map(|d| d.unix_timestamp());
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
            .and_then(|d| OffsetDateTime::parse(d, &Rfc3339).ok())
            .map(|d| d.unix_timestamp())
            .map(|d| d as u64);

        builder.build_get_nym_request(Option::None, &did.id, seq_no, timestamp)
    };
    request
}

pub fn handle_internal_resolution_result(
    namespace: &str,
    ledger_data: &str,
) -> VdrResult<(Result, Metadata)> {
    let (node_response, txn_type, data) = parse_ledger_data(ledger_data)?;
    let txn_type = txn_type
        .as_str()
        .ok_or("Unknown")
        .unwrap()
        .trim_matches('"');
    Ok(match txn_type {
        constants::GET_NYM => {
            let get_nym_result: GetNymResultV1 = serde_json::from_str(data.as_str().unwrap())
                .map_err(|_| err_msg(VdrErrorKind::Resolver, "Could not parse NYM data"))?;

            let did_document = DidDocument::new(
                namespace,
                &get_nym_result.dest,
                &get_nym_result.verkey,
                None,
                get_nym_result
                    .diddoc_content
                    .map(|v| serde_json::from_str(&v).unwrap()),
            );

            let metadata = Metadata::DidDocumentMetadata(DidDocumentMetadata {
                node_response,
                object_type: String::from("NYM"),
                self_certification_version: get_nym_result.version,
            });

            (Result::DidDocument(did_document), metadata)
        }
        constants::GET_CRED_DEF => (
            Result::Content(data),
            Metadata::ContentMetadata(ContentMetadata {
                node_response,
                object_type: String::from("CRED_DEF"),
            }),
        ),
        constants::GET_SCHEMA => (
            Result::Content(data),
            Metadata::ContentMetadata(ContentMetadata {
                node_response,
                object_type: String::from("SCHEMA"),
            }),
        ),
        constants::GET_REVOC_REG_DEF => (
            Result::Content(data),
            Metadata::ContentMetadata(ContentMetadata {
                node_response,
                object_type: String::from("REVOC_REG_DEF"),
            }),
        ),
        constants::GET_REVOC_REG_DELTA => (
            Result::Content(data),
            Metadata::ContentMetadata(ContentMetadata {
                node_response,
                object_type: String::from("REVOC_REG_DELTA"),
            }),
        ),
        _ => (
            Result::Content(data),
            Metadata::ContentMetadata(ContentMetadata {
                node_response,
                object_type: String::from("Unknown"),
            }),
        ),
    })
}

pub fn parse_ledger_data(ledger_data: &str) -> VdrResult<(SJsonValue, SJsonValue, SJsonValue)> {
    let v: SJsonValue = serde_json::from_str(ledger_data)
        .map_err(|_| err_msg(VdrErrorKind::Resolver, "Could not parse ledger response"))?;
    let txn_type = v["result"]["type"].to_owned();
    let data = v["result"]["data"].to_owned();
    if data.is_null() {
        Err(err_msg(VdrErrorKind::Resolver, "Object not found"))
    } else {
        Ok((v, txn_type, data))
    }
}

pub fn parse_or_now(datetime: Option<&String>) -> VdrResult<i64> {
    match datetime {
        Some(datetime) => {
            let dt = OffsetDateTime::parse(datetime, &Rfc3339).map_err(|_| {
                err_msg(
                    VdrErrorKind::Resolver,
                    format!("Could not parse datetime {}", datetime),
                )
            })?;
            Ok(dt.unix_timestamp())
        }
        None => Ok(OffsetDateTime::now_utc().unix_timestamp()),
    }
}

pub async fn handle_request<T: Pool>(pool: &T, request: &PreparedRequest) -> VdrResult<String> {
    let (result, _timing) = request_transaction(pool, request).await?;
    match result {
        RequestResult::Reply(data) => Ok(data),
        RequestResult::Failed(error) => Err(error),
    }
}

pub async fn request_transaction<T: Pool>(
    pool: &T,
    request: &PreparedRequest,
) -> VdrResult<(RequestResult<String>, Option<TimingResult>)> {
    perform_ledger_request(pool, request).await
}

/// Fetch legacy service endpoint using ATTRIB tx
pub async fn fetch_legacy_endpoint<T: Pool>(
    pool: &T,
    did: &DidValue,
    seq_no: Option<i32>,
    timestamp: Option<u64>,
) -> VdrResult<Endpoint> {
    let builder = pool.get_request_builder();
    let request = builder.build_get_attrib_request(
        None,
        did,
        Some(String::from(LEGACY_INDY_SERVICE)),
        None,
        None,
        seq_no,
        timestamp,
    )?;
    debug!(
        "Fetching legacy endpoint for {} with request {:#?}",
        did, request
    );
    let ledger_data = handle_request(pool, &request).await?;
    let (_, _, endpoint_data) = parse_ledger_data(&ledger_data)?;
    let endpoint_data: Endpoint = serde_json::from_str(endpoint_data.as_str().unwrap())
        .map_err(|_| err_msg(VdrErrorKind::Resolver, "Could not parse endpoint data"))?;
    Ok(endpoint_data)
}

#[cfg(test)]
mod tests {

    use percent_encoding::percent_encode;

    use time::format_description::well_known::Rfc3339;
    use time::OffsetDateTime;

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
        let did_url = DidUrl::parse(&did_url_as_str).unwrap();
        let request = build_request(&did_url, &request_builder).unwrap();
        let timestamp = (*(request.req_json).get("operation").unwrap())
            .get("timestamp")
            .unwrap()
            .as_u64()
            .unwrap() as i64;
        assert_eq!(constants::GET_REVOC_REG, request.txn_type);

        assert_eq!(
            OffsetDateTime::parse(datetime_as_str, &Rfc3339)
                .unwrap()
                .unix_timestamp(),
            timestamp
        );
    }

    #[rstest]
    fn build_get_revoc_reg_without_version_time(request_builder: RequestBuilder) {
        let now = OffsetDateTime::now_utc().unix_timestamp();

        let did_url_as_str = "did:indy:idunion:Dk1fRRTtNazyMuK2cr64wp/anoncreds/v0/REV_REG_ENTRY/104/revocable/a4e25e54";
        let did_url = DidUrl::parse(did_url_as_str).unwrap();
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
        let did_url = DidUrl::parse(&did_url_as_str).unwrap();
        let _err = build_request(&did_url, &request_builder).unwrap_err();
    }

    #[rstest]
    fn build_get_revoc_reg_delta_request_with_from_to(request_builder: RequestBuilder) {
        let from_as_str = "2019-12-20T19:17:47Z";
        let to_as_str = "2020-12-20T19:17:47Z";
        let did_url_as_str = format!("did:indy:idunion:Dk1fRRTtNazyMuK2cr64wp/anoncreds/v0/REV_REG_ENTRY/104/revocable/a4e25e54?from={}&to={}",from_as_str, to_as_str);
        let did_url = DidUrl::parse(&did_url_as_str).unwrap();
        let request = build_request(&did_url, &request_builder).unwrap();
        assert_eq!(request.txn_type, constants::GET_REVOC_REG_DELTA);
    }

    #[rstest]
    fn build_get_revoc_reg_delta_request_with_from_only(request_builder: RequestBuilder) {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let from_as_str = "2019-12-20T19:17:47Z";
        let did_url_as_str = format!("did:indy:idunion:Dk1fRRTtNazyMuK2cr64wp/anoncreds/v0/REV_REG_ENTRY/104/revocable/a4e25e54?from={}",from_as_str);
        let did_url = DidUrl::parse(&did_url_as_str).unwrap();
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
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let did_url_as_str = "did:indy:idunion:Dk1fRRTtNazyMuK2cr64wp/anoncreds/v0/REV_REG_DELTA/104/revocable/a4e25e54";
        let did_url = DidUrl::parse(did_url_as_str).unwrap();
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
        let encoded_schema_name =
            percent_encode(name.as_bytes(), percent_encoding::NON_ALPHANUMERIC).to_string();
        let did_url_string = format!(
            "did:indy:idunion:Dk1fRRTtNazyMuK2cr64wp/anoncreds/v0/SCHEMA/{}/1.0",
            encoded_schema_name
        );

        let did_url = DidUrl::parse(did_url_string.as_str()).unwrap();
        let request = build_request(&did_url, &request_builder).unwrap();
        let schema_name = (*(request.req_json).get("operation").unwrap())
            .get("data")
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .unwrap();
        assert_eq!(schema_name, name);
    }
}
