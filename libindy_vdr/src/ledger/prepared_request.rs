use serde_json::{self, Value as SJsonValue};

use super::constants::READ_REQUESTS;
use super::TxnAuthrAgrmtAcceptanceData;
use crate::common::did::DidValue;
use crate::common::error::prelude::*;
use crate::pool::ProtocolVersion;
use crate::state_proof::{
    parse_key_from_request_for_builtin_sp, parse_timestamp_from_req_for_builtin_sp,
};
use crate::utils::base58;
use crate::utils::signature::serialize_signature;

/// A ledger transaction request which has been prepared for dispatch
#[derive(Debug)]
pub struct PreparedRequest {
    /// The protocol version used in pool communication
    pub protocol_version: ProtocolVersion,
    /// The numeric transaction type
    pub txn_type: String,
    /// The numeric transaction request ID
    pub req_id: String,
    /// The request body as a `serde_json::Value` instance
    pub req_json: SJsonValue,
    /// An optional state proof key
    pub sp_key: Option<Vec<u8>>,
    /// Optional state proof timestamps
    pub sp_timestamps: (Option<u64>, Option<u64>),
    /// Mark the request as a read request, which can reduce the number of sockets opened
    pub is_read_request: bool,
}

impl PreparedRequest {
    /// Create a new `PreparedRequest`
    pub fn new(
        protocol_version: ProtocolVersion,
        txn_type: String,
        req_id: String,
        req_json: SJsonValue,
        sp_key: Option<Vec<u8>>,
        sp_timestamps: (Option<u64>, Option<u64>),
        is_read_request: bool,
    ) -> Self {
        Self {
            protocol_version,
            txn_type,
            req_id,
            req_json,
            sp_key,
            sp_timestamps,
            is_read_request,
        }
    }

    /// Generate the normalized representation of a transaction for signing the request
    pub fn get_signature_input(&self) -> VdrResult<String> {
        Ok(serialize_signature(&self.req_json)?)
    }

    /// Assign the endorser property of the prepared request
    pub fn set_endorser(&mut self, endorser: &DidValue) -> VdrResult<()> {
        self.req_json["endorser"] = SJsonValue::String(endorser.to_short().to_string());
        Ok(())
    }

    /// Assign the signature property of the prepared request
    pub fn set_signature(&mut self, signature: &[u8]) -> VdrResult<()> {
        self.req_json["signature"] = SJsonValue::String(base58::encode(signature));
        Ok(())
    }

    /// Add a signature to the prepared request
    pub fn set_multi_signature(
        &mut self,
        identifier: &DidValue,
        signature: &[u8],
    ) -> VdrResult<()> {
        self.req_json.as_object_mut().map(|request| {
            if !request.contains_key("signatures") {
                request.insert(
                    "signatures".to_string(),
                    serde_json::Value::Object(serde_json::Map::new()),
                );
            }
            request["signatures"]
                .as_object_mut()
                .unwrap()
                .insert(identifier.0.to_owned(), json!(base58::encode(signature)));

            if let (Some(identifier), Some(signature)) = (
                request
                    .get("identifier")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned),
                request.remove("signature"),
            ) {
                request["signatures"]
                    .as_object_mut()
                    .unwrap()
                    .insert(identifier, signature);
            }
        });

        Ok(())
    }

    /// Decorate the prepared request with the transaction author agreement acceptance
    pub fn set_txn_author_agreement_acceptance(
        &mut self,
        acceptance: &TxnAuthrAgrmtAcceptanceData,
    ) -> VdrResult<()> {
        self.req_json["taaAcceptance"] = serde_json::to_value(acceptance)
            .with_err_msg(VdrErrorKind::Unexpected, "Error serializing TAA acceptance")?;
        Ok(())
    }

    /// Construct a prepared request from user-provided JSON
    pub fn from_request_json(message: &str) -> VdrResult<PreparedRequest> {
        let req_json: SJsonValue =
            serde_json::from_str(message).with_input_err("Invalid request JSON")?;

        let protocol_version = req_json["protocolVersion"]
            .as_u64()
            .ok_or(input_err("Invalid request JSON: protocolVersion not found"))
            .and_then(ProtocolVersion::from_id)?;

        let req_id = req_json["reqId"]
            .as_u64()
            .ok_or(input_err("Invalid request JSON: reqId not found"))?
            .to_string();

        let txn_type = req_json["operation"]["type"]
            .as_str()
            .ok_or_else(|| input_err("No operation type in request"))?
            .to_string();

        let (sp_key, sp_timestamps) = (
            parse_key_from_request_for_builtin_sp(&req_json, protocol_version),
            parse_timestamp_from_req_for_builtin_sp(&req_json, txn_type.as_str()),
        );

        let is_read_request = sp_key.is_some() || READ_REQUESTS.contains(&txn_type.as_str());

        Ok(PreparedRequest::new(
            protocol_version,
            txn_type,
            req_id,
            req_json,
            sp_key,
            sp_timestamps,
            is_read_request,
        ))
    }
}
