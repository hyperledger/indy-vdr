use serde_json::Value as SJsonValue;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use super::did::{DidUrl, LedgerObject, QueryParameter};

use crate::common::error::prelude::*;
use crate::ledger::identifiers::{CredentialDefinitionId, RevocationRegistryId, SchemaId};
use crate::ledger::RequestBuilder;
use crate::pool::PreparedRequest;
use crate::utils::Qualifiable;

pub fn build_request(did: &DidUrl, builder: &RequestBuilder) -> VdrResult<PreparedRequest> {
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
                            .and_then(|d| OffsetDateTime::parse(d, &Rfc3339).ok())
                            .and_then(|d| Some(d.unix_timestamp()));
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
                        .and_then(|d| Some(d.unix_timestamp()));
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
            .and_then(|d| Some(d.unix_timestamp()))
            .and_then(|d| Some(d as u64));

        builder.build_get_nym_request(Option::None, &did.id, seq_no, timestamp)
    };
    request
}

pub fn parse_ledger_data(ledger_data: &str) -> VdrResult<SJsonValue> {
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
