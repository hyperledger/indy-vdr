use serde_json::{self, Value as SJsonValue};

use super::new_request_id;
use crate::common::error::prelude::*;
use crate::ledger::constants::READ_REQUESTS;
use crate::ledger::TxnAuthrAgrmtAcceptanceData;
use crate::pool::ProtocolVersion;
use crate::state_proof::{
    constants::REQUEST_FOR_FULL, parse_key_from_request_for_builtin_sp,
    parse_timestamp_from_req_for_builtin_sp, BoxedSPParser,
};
use crate::utils::base58;
use crate::utils::did::{DidValue, DEFAULT_LIBINDY_DID};
use crate::utils::txn_signature::serialize_signature;
use crate::utils::Validatable;

/// Determines the handler and state proof semantics used to process a request
#[derive(PartialEq, Eq)]
pub enum RequestMethod {
    Consensus,
    ReadConsensus,
    BuiltinStateProof {
        sp_key: Vec<u8>,
        sp_timestamps: (Option<u64>, Option<u64>),
    },
    CustomStateProof {
        sp_parser: BoxedSPParser,
        sp_timestamps: (Option<u64>, Option<u64>),
    },
    Full {
        node_aliases: Option<Vec<String>>,
        timeout: Option<i64>,
    },
}

impl std::fmt::Debug for RequestMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let desc = match self {
            Self::Consensus => "Consensus",
            Self::ReadConsensus => "ReadConsensus",
            Self::BuiltinStateProof { .. } => "BuiltinStateProof",
            Self::CustomStateProof { .. } => "CustomStateProof",
            Self::Full { .. } => "Full",
        };
        write!(f, "RequestMethod({})", desc)
    }
}

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
    /// Determine the request handler to use
    pub method: RequestMethod,
}

impl PreparedRequest {
    /// Create a new `PreparedRequest`
    pub fn new(
        protocol_version: ProtocolVersion,
        txn_type: String,
        req_id: String,
        req_json: SJsonValue,
        method: Option<RequestMethod>,
    ) -> Self {
        let method = method.unwrap_or_else(|| Self::default_method(txn_type.as_str()));
        Self {
            protocol_version,
            txn_type,
            req_id,
            req_json,
            method,
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
        if let Some(request) = self.req_json.as_object_mut() {
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
        }

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
    pub fn from_request_json<T: AsRef<[u8]>>(message: T) -> VdrResult<PreparedRequest> {
        let req_json: SJsonValue =
            serde_json::from_slice(message.as_ref()).with_input_err("Invalid request JSON")?;
        Self::from_request_json_ext(req_json, false, None)
    }

    /// Construct a prepared request from user-provided JSON
    pub fn from_request_json_ext(
        mut req_json: SJsonValue,
        auto_pop: bool,
        method: Option<RequestMethod>,
    ) -> VdrResult<PreparedRequest> {
        let protocol_version = req_json["protocolVersion"]
            .as_i64()
            .ok_or_else(|| input_err("Invalid request JSON: protocolVersion not found"))
            .and_then(ProtocolVersion::from_id)?;

        let req_id = req_json["reqId"].as_i64();
        let req_id = if let Some(req_id) = req_id {
            req_id
        } else if auto_pop {
            let new_req_id = new_request_id();
            req_json["reqId"] = SJsonValue::from(new_req_id);
            new_req_id
        } else {
            return Err(input_err("Invalid request JSON: reqId not found"));
        }
        .to_string();

        if let Some(ident) = req_json["identifier"].as_str() {
            DidValue(ident.to_owned()).validate()?
        } else if auto_pop {
            req_json["identifier"] = SJsonValue::from(DEFAULT_LIBINDY_DID.to_string());
        } else {
            return Err(input_err("Invalid request JSON: missing identifier DID"));
        }

        let txn_type = req_json["operation"]["type"]
            .as_str()
            .ok_or_else(|| input_err("No operation type in request"))?
            .to_string();

        let method = if method.is_some() {
            method
        } else {
            let (sp_key, sp_timestamps) = (
                parse_key_from_request_for_builtin_sp(&req_json, protocol_version),
                parse_timestamp_from_req_for_builtin_sp(&req_json, txn_type.as_str()),
            );

            sp_key.map(|sp_key| RequestMethod::BuiltinStateProof {
                sp_key,
                sp_timestamps,
            })
        };

        Ok(Self::new(
            protocol_version,
            txn_type,
            req_id,
            req_json,
            method,
        ))
    }

    fn default_method(txn_type: &str) -> RequestMethod {
        if REQUEST_FOR_FULL.contains(&txn_type) {
            RequestMethod::Full {
                node_aliases: None,
                timeout: None,
            }
        } else if READ_REQUESTS.contains(&txn_type) {
            RequestMethod::ReadConsensus
        } else {
            RequestMethod::Consensus
        }
    }
}
