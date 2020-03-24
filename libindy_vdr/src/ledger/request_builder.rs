use hex::FromHex;
use serde_json::{self, Value as SJsonValue};

use crate::common::did::{DidValue, DEFAULT_LIBINDY_DID};
use crate::common::error::prelude::*;
use crate::pool::{ProtocolVersion, RequestTarget};
use crate::state_proof::{
    constants::REQUEST_FOR_FULL, parse_key_from_request_for_builtin_sp,
    parse_timestamp_from_req_for_builtin_sp,
};
use crate::utils::base58::ToBase58;
use crate::utils::hash::{digest, Sha256};
use crate::utils::qualifier::Qualifiable;
use crate::utils::signature::serialize_signature;

use super::identifiers::cred_def::CredentialDefinitionId;
use super::identifiers::rev_reg::RevocationRegistryId;
use super::identifiers::rich_schema::RichSchemaId;
use super::identifiers::schema::SchemaId;
use super::requests::attrib::{AttribOperation, GetAttribOperation};
use super::requests::auth_rule::*;
use super::requests::author_agreement::*;
use super::requests::cred_def::{CredDefOperation, CredentialDefinition, GetCredDefOperation};
use super::requests::node::{NodeOperation, NodeOperationData};
use super::requests::nym::{role_to_code, GetNymOperation, NymOperation};
use super::requests::pool::{
    PoolConfigOperation, PoolRestartOperation, PoolUpgradeOperation, Schedule,
};
use super::requests::rev_reg::{
    GetRevRegDeltaOperation, GetRevRegOperation, RevRegEntryOperation, RevocationRegistryDelta,
};
use super::requests::rev_reg_def::{
    GetRevRegDefOperation, RegistryType, RevRegDefOperation, RevocationRegistryDefinition,
};
use super::requests::schema::{
    GetSchemaOperation, GetSchemaOperationData, Schema, SchemaOperation, SchemaOperationData,
};
use super::requests::txn::GetTxnOperation;
use super::requests::validator_info::GetValidatorInfoOperation;
use super::requests::{get_request_id, Request, RequestType, TxnAuthrAgrmtAcceptanceData};

use super::constants::{txn_name_to_code, READ_REQUESTS};
use crate::ledger::requests::rich_schema::{GetRichSchemaById, GetRichSchemaByIdOperation, GetRichSchemaByMetadata, GetRichSchemaByMetadataOperation, RSContent, RichSchema, RichSchemaOperation, RSType, RSContextOperation, RSMappingOperation, RSEncodingOperation, RSCredDefOperation, RSPresDefOperation};

fn datetime_to_date_timestamp(time: u64) -> u64 {
    const SEC_IN_DAY: u64 = 86400;
    time / SEC_IN_DAY * SEC_IN_DAY
}

fn calculate_hash(text: &str, version: &str) -> VdrResult<Vec<u8>> {
    let content: String = version.to_string() + text;
    Ok(digest::<Sha256>(content.as_bytes()))
}

fn compare_hash(text: &str, version: &str, hash: &str) -> VdrResult<()> {
    let calculated_hash = calculate_hash(text, version)?;

    let passed_hash = Vec::from_hex(hash).with_input_err("Cannot decode hash")?;

    if calculated_hash != passed_hash {
        return Err(input_err(
            format!("Calculated hash of concatenation `version` and `text` doesn't equal to passed `hash` value. \n\
            Calculated hash value: {:?}, \n Passed hash value: {:?}", calculated_hash, passed_hash)));
    }
    Ok(())
}

#[derive(Debug)]
pub struct PreparedRequest {
    pub protocol_version: ProtocolVersion,
    pub txn_type: String,
    pub req_id: String,
    pub req_json: SJsonValue,
    pub sp_key: Option<Vec<u8>>,
    pub sp_timestamps: (Option<u64>, Option<u64>),
    pub is_read_request: bool,
}

impl PreparedRequest {
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

    pub fn get_signature_input(&self) -> VdrResult<String> {
        Ok(serialize_signature(&self.req_json)?)
    }

    pub fn set_endorser(&mut self, endorser: &DidValue) -> VdrResult<()> {
        self.req_json["endorser"] = SJsonValue::String(endorser.to_short().to_string());
        Ok(())
    }

    pub fn set_signature(&mut self, signature: &[u8]) -> VdrResult<()> {
        self.req_json["signature"] = SJsonValue::String(signature.to_base58());
        Ok(())
    }

    pub fn set_txn_author_agreement_acceptance(
        &mut self,
        acceptance: &TxnAuthrAgrmtAcceptanceData,
    ) -> VdrResult<()> {
        self.req_json["taaAcceptance"] = serde_json::to_value(acceptance)
            .with_err_msg(VdrErrorKind::Unexpected, "Error serializing TAA acceptance")?;
        Ok(())
    }

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

pub struct RequestBuilder {
    pub protocol_version: ProtocolVersion,
}

impl Default for RequestBuilder {
    fn default() -> Self {
        Self::new(ProtocolVersion::default())
    }
}

impl RequestBuilder {
    pub fn new(protocol_version: ProtocolVersion) -> Self {
        Self { protocol_version }
    }

    pub fn build<T: RequestType>(
        &self,
        operation: T,
        identifier: Option<&DidValue>,
    ) -> VdrResult<PreparedRequest> {
        let req_id = get_request_id();
        let identifier = identifier.or(Some(&DEFAULT_LIBINDY_DID));
        let txn_type = T::get_txn_type().to_string();
        let sp_key = operation.get_sp_key(self.protocol_version)?;
        let sp_timestamps = operation.get_sp_timestamps()?;
        let is_read_request = sp_key.is_some() || READ_REQUESTS.contains(&txn_type.as_str());
        let body = Request::build_request(
            req_id,
            operation,
            identifier,
            Some(self.protocol_version as usize),
        )?;
        trace!("Prepared request: {} {}", req_id, body);
        Ok(PreparedRequest::new(
            self.protocol_version,
            txn_type,
            req_id.to_string(),
            body,
            sp_key,
            sp_timestamps,
            is_read_request,
        ))
    }

    pub fn build_nym_request(
        &self,
        identifier: &DidValue,
        dest: &DidValue,
        verkey: Option<String>,
        alias: Option<String>,
        role: Option<String>,
    ) -> VdrResult<PreparedRequest> {
        let role = role_to_code(role)?;
        let operation = NymOperation::new(dest.to_short(), verkey, alias, role);
        self.build(operation, Some(identifier))
    }

    pub fn build_get_nym_request(
        &self,
        identifier: Option<&DidValue>,
        dest: &DidValue,
    ) -> VdrResult<PreparedRequest> {
        let dest = dest.to_short();
        let operation = GetNymOperation::new(dest.clone());
        self.build(operation, identifier)
    }

    pub fn build_attrib_request(
        &self,
        identifier: &DidValue,
        dest: &DidValue,
        hash: Option<String>,
        raw: Option<&SJsonValue>,
        enc: Option<String>,
    ) -> VdrResult<PreparedRequest> {
        let operation =
            AttribOperation::new(dest.to_short(), hash, raw.map(SJsonValue::to_string), enc);
        self.build(operation, Some(identifier))
    }

    pub fn build_get_attrib_request(
        &self,
        identifier: Option<&DidValue>,
        dest: &DidValue,
        raw: Option<String>,
        hash: Option<String>,
        enc: Option<String>,
    ) -> VdrResult<PreparedRequest> {
        let operation = GetAttribOperation::new(dest.to_short(), raw, hash, enc);
        self.build(operation, identifier)
    }

    pub fn build_node_request(
        &self,
        identifier: &DidValue,
        dest: &DidValue,
        data: NodeOperationData,
    ) -> VdrResult<PreparedRequest> {
        let operation = NodeOperation::new(dest.to_short(), data);
        self.build(operation, Some(identifier))
    }

    pub fn build_get_validator_info_request(
        &self,
        identifier: &DidValue,
    ) -> VdrResult<PreparedRequest> {
        self.build(GetValidatorInfoOperation::new(), Some(identifier))
    }

    pub fn build_get_txn_request(
        &self,
        identifier: Option<&DidValue>,
        ledger_type: i32,
        seq_no: i32,
    ) -> VdrResult<PreparedRequest> {
        if seq_no <= 0 {
            return Err(input_err("Transaction number must be > 0"));
        }
        self.build(GetTxnOperation::new(seq_no, ledger_type), identifier)
    }

    pub fn build_pool_config(
        &self,
        identifier: &DidValue,
        writes: bool,
        force: bool,
    ) -> VdrResult<PreparedRequest> {
        self.build(PoolConfigOperation::new(writes, force), Some(identifier))
    }

    pub fn build_pool_restart(
        &self,
        identifier: &DidValue,
        action: &str,
        datetime: Option<&str>,
    ) -> VdrResult<PreparedRequest> {
        self.build(
            PoolRestartOperation::new(action, datetime.map(String::from)),
            Some(identifier),
        )
    }

    pub fn build_pool_upgrade(
        &self,
        identifier: &DidValue,
        name: &str,
        version: &str,
        action: &str,
        sha256: &str,
        timeout: Option<u32>,
        schedule: Option<Schedule>,
        justification: Option<&str>,
        reinstall: bool,
        force: bool,
        package: Option<&str>,
    ) -> VdrResult<PreparedRequest> {
        let operation = PoolUpgradeOperation::new(
            name,
            version,
            action,
            sha256,
            timeout,
            schedule,
            justification,
            reinstall,
            force,
            package,
        );
        self.build(operation, Some(identifier))
    }

    pub fn build_auth_rule_request(
        &self,
        submitter_did: &DidValue,
        txn_type: String,
        action: String,
        field: String,
        old_value: Option<String>,
        new_value: Option<String>,
        constraint: Constraint,
    ) -> VdrResult<PreparedRequest> {
        let txn_type = txn_name_to_code(&txn_type)
            .ok_or_else(|| input_err(format!("Unsupported `txn_type`: {}", txn_type)))?
            .to_string();

        let action = serde_json::from_str::<AuthAction>(&format!("\"{}\"", action))
            .map_err(|err| input_err(format!("Cannot parse auth action: {}", err)))?;

        let operation =
            AuthRuleOperation::new(txn_type, field, action, old_value, new_value, constraint);
        self.build(operation, Some(submitter_did))
    }

    pub fn build_auth_rules_request(
        &self,
        submitter_did: &DidValue,
        rules: AuthRules,
    ) -> VdrResult<PreparedRequest> {
        self.build(AuthRulesOperation::new(rules), Some(submitter_did))
    }

    pub fn build_get_auth_rule_request(
        &self,
        submitter_did: Option<&DidValue>,
        auth_type: Option<String>,
        auth_action: Option<String>,
        field: Option<String>,
        old_value: Option<String>,
        new_value: Option<String>,
    ) -> VdrResult<PreparedRequest> {
        let operation = match (auth_type, auth_action, field) {
            (None, None, None) => GetAuthRuleOperation::get_all(),
            (Some(auth_type), Some(auth_action), Some(field)) => {
                let type_ = txn_name_to_code(&auth_type)
                    .ok_or_else(|| input_err(format!("Unsupported `auth_type`: {}", auth_type)))?;

                let action = serde_json::from_str::<AuthAction>(&format!("\"{}\"", auth_action))
                    .map_err(|err| input_err(format!("Cannot parse auth action: {}", err)))?;

                GetAuthRuleOperation::get_one(
                    type_.to_string(),
                    field,
                    action,
                    old_value.map(String::from),
                    new_value.map(String::from),
                )
            }
            _ => {
                return Err(input_err(
                    "Either none or all transaction related parameters must be specified.",
                ))
            }
        };
        self.build(operation, submitter_did)
    }

    pub fn build_txn_author_agreement_request(
        &self,
        identifier: &DidValue,
        text: Option<String>,
        version: String,
        ratification_ts: Option<u64>,
        retirement_ts: Option<u64>,
    ) -> VdrResult<PreparedRequest> {
        self.build(
            TxnAuthorAgreementOperation::new(text, version, ratification_ts, retirement_ts),
            Some(identifier),
        )
    }

    pub fn build_get_txn_author_agreement_request(
        &self,
        identifier: Option<&DidValue>,
        data: Option<&GetTxnAuthorAgreementData>,
    ) -> VdrResult<PreparedRequest> {
        self.build(GetTxnAuthorAgreementOperation::new(data), identifier)
    }

    pub fn build_disable_all_txn_author_agreements_request(
        &self,
        identifier: &DidValue,
    ) -> VdrResult<PreparedRequest> {
        self.build(
            DisableAllTxnAuthorAgreementsOperation::new(),
            Some(identifier),
        )
    }

    pub fn build_acceptance_mechanisms_request(
        &self,
        identifier: &DidValue,
        aml: AcceptanceMechanisms,
        version: String,
        aml_context: Option<String>,
    ) -> VdrResult<PreparedRequest> {
        let operation = SetAcceptanceMechanismOperation::new(
            aml,
            version.to_string(),
            aml_context.map(String::from),
        );
        self.build(operation, Some(identifier))
    }

    pub fn build_get_acceptance_mechanisms_request(
        &self,
        identifier: Option<&DidValue>,
        timestamp: Option<u64>,
        version: Option<String>,
    ) -> VdrResult<PreparedRequest> {
        if timestamp.is_some() && version.is_some() {
            return Err(input_err(
                "timestamp and version cannot be specified together.",
            ));
        }
        self.build(
            GetAcceptanceMechanismOperation::new(timestamp, version.map(String::from)),
            identifier,
        )
    }

    pub fn build_schema_request(
        &self,
        identifier: &DidValue,
        schema: Schema,
    ) -> VdrResult<PreparedRequest> {
        let schema = match schema {
            Schema::SchemaV1(s) => s,
        };
        let schema_data =
            SchemaOperationData::new(schema.name, schema.version, schema.attr_names.into());
        self.build(SchemaOperation::new(schema_data), Some(identifier))
    }

    pub fn build_get_schema_request(
        &self,
        identifier: Option<&DidValue>,
        id: &SchemaId,
    ) -> VdrResult<PreparedRequest> {
        let id = id.to_unqualified();
        let (_, dest, name, version) = id.parts().ok_or(input_err(format!(
            "Schema ID `{}` cannot be used to build request: invalid number of parts",
            id.0
        )))?;
        let data = GetSchemaOperationData::new(name, version);
        self.build(GetSchemaOperation::new(dest.to_short(), data), identifier)
    }

    pub fn build_cred_def_request(
        &self,
        identifier: &DidValue,
        cred_def: CredentialDefinition,
    ) -> VdrResult<PreparedRequest> {
        let cred_def = match cred_def.to_unqualified() {
            CredentialDefinition::CredentialDefinitionV1(cred_def) => cred_def,
        };
        self.build(CredDefOperation::new(cred_def), Some(identifier))
    }

    pub fn build_get_cred_def_request(
        &self,
        identifier: Option<&DidValue>,
        id: &CredentialDefinitionId,
    ) -> VdrResult<PreparedRequest> {
        let id = id.to_unqualified();
        let (_, origin, signature_type, schema_id, tag) = id.parts()
            .ok_or_else(|| input_err(format!("Credential Definition ID `{}` cannot be used to build request: invalid number of parts", id.0)))?;

        let ref_ = schema_id
            .0
            .parse::<i32>()
            .map_input_err(|| format!("Schema ID is invalid number in: {:?}", id))?;
        let operation =
            GetCredDefOperation::new(ref_, signature_type, origin.to_short(), Some(tag));
        self.build(operation, identifier)
    }

    pub fn build_get_revoc_reg_def_request(
        &self,
        identifier: Option<&DidValue>,
        id: &RevocationRegistryId,
    ) -> VdrResult<PreparedRequest> {
        let id = id.to_unqualified();
        self.build(GetRevRegDefOperation::new(&id), identifier)
    }

    pub fn build_get_revoc_reg_request(
        &self,
        identifier: Option<&DidValue>,
        revoc_reg_def_id: &RevocationRegistryId,
        timestamp: i64,
    ) -> VdrResult<PreparedRequest> {
        let revoc_reg_def_id = revoc_reg_def_id.to_unqualified();
        self.build(
            GetRevRegOperation::new(&revoc_reg_def_id, timestamp),
            identifier,
        )
    }

    pub fn build_get_revoc_reg_delta_request(
        &self,
        identifier: Option<&DidValue>,
        revoc_reg_def_id: &RevocationRegistryId,
        from: Option<i64>,
        to: i64,
    ) -> VdrResult<PreparedRequest> {
        let revoc_reg_def_id = revoc_reg_def_id.to_unqualified();
        self.build(
            GetRevRegDeltaOperation::new(&revoc_reg_def_id, from, to),
            identifier,
        )
    }

    pub fn build_revoc_reg_def_request(
        &self,
        identifier: &DidValue,
        revoc_reg: RevocationRegistryDefinition,
    ) -> VdrResult<PreparedRequest> {
        let revoc_reg = match revoc_reg.to_unqualified() {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(revoc_reg) => revoc_reg,
        };
        self.build(RevRegDefOperation::new(revoc_reg), Some(identifier))
    }

    pub fn build_revoc_reg_entry_request(
        &self,
        identifier: &DidValue,
        revoc_reg_def_id: &RevocationRegistryId,
        revoc_def_type: &RegistryType,
        rev_reg_entry: RevocationRegistryDelta,
    ) -> VdrResult<PreparedRequest> {
        let revoc_reg_def_id = revoc_reg_def_id.to_unqualified();
        let rev_reg_entry = match rev_reg_entry {
            RevocationRegistryDelta::RevocationRegistryDeltaV1(entry) => entry,
        };
        self.build(
            RevRegEntryOperation::new(revoc_def_type, &revoc_reg_def_id, rev_reg_entry),
            Some(identifier),
        )
    }

    pub fn prepare_txn_author_agreement_acceptance_data(
        &self,
        text: Option<&str>,
        version: Option<&str>,
        hash: Option<&str>,
        mechanism: &str,
        time: u64,
    ) -> VdrResult<TxnAuthrAgrmtAcceptanceData> {
        let taa_digest = match (text, version, hash) {
            (None, None, None) => {
                return Err(input_err("Invalid combination of params: Either combination `text` + `version` or `taa_digest` must be passed."));
            }
            (None, None, Some(hash_)) => hash_.to_string(),
            (Some(_), None, _) | (None, Some(_), _) => {
                return Err(input_err("Invalid combination of params: `text` and `version` should be passed or skipped together."));
            }
            (Some(text_), Some(version_), None) => hex::encode(calculate_hash(text_, version_)?),
            (Some(text_), Some(version_), Some(hash_)) => {
                compare_hash(text_, version_, hash_)?;
                hash_.to_string()
            }
        };

        let acceptance_data = TxnAuthrAgrmtAcceptanceData {
            mechanism: mechanism.to_string(),
            taa_digest,
            time: datetime_to_date_timestamp(time),
        };

        Ok(acceptance_data)
    }

    pub fn build_custom_request(
        &self,
        message: &[u8],
        sp_key: Option<Vec<u8>>,
        sp_timestamps: (Option<u64>, Option<u64>),
    ) -> VdrResult<(PreparedRequest, Option<RequestTarget>)> {
        let message = std::str::from_utf8(message).with_input_err("Invalid UTF-8")?;
        self.build_custom_request_from_str(message, sp_key, sp_timestamps)
    }

    pub fn build_custom_request_from_str(
        &self,
        message: &str,
        sp_key: Option<Vec<u8>>,
        sp_timestamps: (Option<u64>, Option<u64>),
    ) -> VdrResult<(PreparedRequest, Option<RequestTarget>)> {
        let mut req_json: SJsonValue =
            serde_json::from_str(message).with_input_err("Invalid request JSON")?;

        let protocol_version = req_json["protocolVersion"].as_u64();
        let protocol_version = if protocol_version.is_none() {
            req_json["protocolVersion"] = SJsonValue::from(self.protocol_version as usize);
            self.protocol_version
        } else {
            ProtocolVersion::from_id(protocol_version.unwrap())?
        };

        let ident = req_json["identifier"].as_str();
        if ident.is_none() {
            req_json["identifier"] = SJsonValue::from(DEFAULT_LIBINDY_DID.to_string());
        } else {
            // FIXME validate identifier
        }

        let req_id = req_json["reqId"].as_u64();
        let req_id = if req_id.is_none() {
            let new_req_id = get_request_id();
            req_json["reqId"] = SJsonValue::from(new_req_id);
            new_req_id
        } else {
            req_id.unwrap() // FIXME validate?
        }
        .to_string();

        let txn_type = req_json["operation"]["type"]
            .as_str()
            .ok_or_else(|| input_err("No operation type in request"))?
            .to_string();

        let target = if REQUEST_FOR_FULL.contains(&txn_type.as_str()) {
            Some(RequestTarget::Full(None, None))
        } else {
            None
        };

        let (sp_key, sp_timestamps) = if sp_key.is_some() {
            (sp_key, sp_timestamps)
        } else {
            (
                parse_key_from_request_for_builtin_sp(&req_json, protocol_version),
                parse_timestamp_from_req_for_builtin_sp(&req_json, txn_type.as_str()),
            )
        };
        let is_read_request = sp_key.is_some() || READ_REQUESTS.contains(&txn_type.as_str());
        Ok((
            PreparedRequest::new(
                protocol_version,
                txn_type,
                req_id,
                req_json,
                sp_key,
                sp_timestamps,
                is_read_request,
            ),
            target,
        ))
    }
    pub fn build_rich_schema_request(
        &self,
        identifier: &DidValue,
        id: RichSchemaId,
        content: RSContent,
        rs_name: String,
        rs_version: String,
        rs_type: String,
        ver: String,
    ) -> VdrResult<PreparedRequest> {
        let rich_schema = RichSchema::new(id, content, rs_name, rs_version, rs_type.clone(), ver);
        // let rs_op_type: &RichSchemaOperation = RS_TYPE_TO_OP_STRUCT.get(rs_type.as_str())?;
        let mut general_op = RichSchemaOperation::new(rich_schema);
        match serde_json::from_value(serde_json::value::Value::String(rs_type))? {
            RSType::Sch => {
                self.build(general_op, Some(identifier))
            },
            RSType::Map => {
                general_op._type = RSMappingOperation::get_txn_type().to_string();
                self.build(RSMappingOperation(general_op), Some(identifier))
            },
            RSType::Enc => {
                general_op._type = RSEncodingOperation::get_txn_type().to_string();
                self.build(RSEncodingOperation(general_op), Some(identifier))
            }
            RSType::Ctx => {
                general_op._type = RSContextOperation::get_txn_type().to_string();
                self.build(RSContextOperation(general_op), Some(identifier))
            },
            RSType::Cdf => {
                general_op._type = RSCredDefOperation::get_txn_type().to_string();
                self.build(RSCredDefOperation(general_op), Some(identifier))
            },
            RSType::Pdf => {
                general_op._type = RSPresDefOperation::get_txn_type().to_string();
                self.build(RSPresDefOperation(general_op), Some(identifier))
            },
        }
    }
    pub fn build_get_rich_schema_by_id(
        &self,
        identifier: &DidValue,
        rs_id: &RichSchemaId,
    ) -> VdrResult<PreparedRequest> {
        let get_rs_by_id: GetRichSchemaById = GetRichSchemaById::new(rs_id.to_unqualified());
        self.build(
            GetRichSchemaByIdOperation::new(get_rs_by_id),
            Some(identifier),
        )
    }
    pub fn build_get_rich_schema_by_metadata(
        &self,
        identifier: &DidValue,
        rs_type: String,
        rs_name: String,
        rs_version: String,
    ) -> VdrResult<PreparedRequest> {
        let get_rs_by_meta: GetRichSchemaByMetadata =
            GetRichSchemaByMetadata::new(rs_type, rs_name, rs_version);
        self.build(
            GetRichSchemaByMetadataOperation::new(get_rs_by_meta),
            Some(identifier),
        )
    }
}

/*
#[cfg(test)]
mod tests {
    use self::domain::node::Services;

    const IDENTIFIER: &str = "NcYxiDXkpYi6ov5FcYDi1e";
    const DEST: &str = "VsKV7grR1BUE29mG2Fm2kX";
    const VERKEY: &str = "CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW";

    fn identifier() -> DidValue {
        DidValue(IDENTIFIER.to_string())
    }

    fn dest() -> DidValue {
        DidValue(DEST.to_string())
    }

    #[test]
    fn build_nym_request_works_for_only_required_fields() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": NYM,
            "dest": DEST
        });

        let request = ledger_service
            .build_nym_request(&identifier(), &dest(), None, None, None)
            .unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_nym_request_works_for_empty_role() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": NYM,
            "dest": DEST,
            "role": SJsonValue::Null,
        });

        let request = ledger_service
            .build_nym_request(&identifier(), &dest(), None, None, Some(""))
            .unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_nym_request_works_for_optional_fields() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": NYM,
            "dest": DEST,
            "role": SJsonValue::Null,
            "alias": "some_alias",
            "verkey": VERKEY,
        });

        let request = ledger_service
            .build_nym_request(
                &identifier(),
                &dest(),
                Some(VERKEY),
                Some("some_alias"),
                Some(""),
            )
            .unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_nym_request_works() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": GET_NYM,
            "dest": DEST
        });

        let request = ledger_service
            .build_get_nym_request(Some(&identifier()), &dest())
            .unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_ddo_request_works() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": GET_DDO,
            "dest": DEST
        });

        let request = ledger_service
            .build_get_ddo_request(Some(&identifier()), &dest())
            .unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_attrib_request_works_for_hash_field() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": ATTRIB,
            "dest": DEST,
            "hash": "hash"
        });

        let request = ledger_service
            .build_attrib_request(&identifier(), &dest(), Some("hash"), None, None)
            .unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_attrib_request_works_for_raw_value() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": GET_ATTR,
            "dest": DEST,
            "raw": "raw"
        });

        let request = ledger_service
            .build_get_attrib_request(Some(&identifier()), &dest(), Some("raw"), None, None)
            .unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_attrib_request_works_for_hash_value() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": GET_ATTR,
            "dest": DEST,
            "hash": "hash"
        });

        let request = ledger_service
            .build_get_attrib_request(Some(&identifier()), &dest(), None, Some("hash"), None)
            .unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_attrib_request_works_for_enc_value() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": GET_ATTR,
            "dest": DEST,
            "enc": "enc"
        });

        let request = ledger_service
            .build_get_attrib_request(Some(&identifier()), &dest(), None, None, Some("enc"))
            .unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_node_request_works() {
        let ledger_service = LedgerService::new();

        let data = NodeOperationData {
            node_ip: Some("ip".to_string()),
            node_port: Some(1),
            client_ip: Some("ip".to_string()),
            client_port: Some(1),
            alias: "some".to_string(),
            services: Some(vec![Services::VALIDATOR]),
            blskey: Some("blskey".to_string()),
            blskey_pop: Some("pop".to_string()),
        };

        let expected_result = json!({
            "type": NODE,
            "dest": DEST,
            "data": {
                "node_ip": "ip",
                "node_port": 1,
                "client_ip": "ip",
                "client_port": 1,
                "alias": "some",
                "services": ["VALIDATOR"],
                "blskey": "blskey",
                "blskey_pop": "pop"
            }
        });

        let request = ledger_service
            .build_node_request(&identifier(), &dest(), data)
            .unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_txn_request_works() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": GET_TXN,
            "data": 1,
            "ledgerId": 1
        });

        let request = ledger_service
            .build_get_txn_request(Some(&identifier()), None, 1)
            .unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_txn_request_works_for_ledger_type_as_predefined_string_constant() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": GET_TXN,
            "data": 1,
            "ledgerId": 0
        });

        let request = ledger_service
            .build_get_txn_request(Some(&identifier()), Some("POOL"), 1)
            .unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_txn_request_works_for_ledger_type_as_number() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": GET_TXN,
            "data": 1,
            "ledgerId": 10
        });

        let request = ledger_service
            .build_get_txn_request(Some(&identifier()), Some("10"), 1)
            .unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_txn_request_works_for_invalid_type() {
        let ledger_service = LedgerService::new();

        let res = ledger_service.build_get_txn_request(Some(&identifier()), Some("type"), 1);
        assert_kind!(VdrErrorKind::InvalidStructure, res);
    }

    #[test]
    fn validate_action_works_for_pool_restart() {
        let ledger_service = LedgerService::new();
        let request = ledger_service
            .build_pool_restart(&identifier(), "start", None)
            .unwrap();
        ledger_service.validate_action(&request).unwrap();
    }

    #[test]
    fn validate_action_works_for_get_validator_info() {
        let ledger_service = LedgerService::new();
        let request = ledger_service
            .build_get_validator_info_request(&identifier())
            .unwrap();
        ledger_service.validate_action(&request).unwrap();
    }

    mod auth_rule {
        use super::*;

        const ADD_AUTH_ACTION: &str = "ADD";
        const EDIT_AUTH_ACTION: &str = "EDIT";
        const FIELD: &str = "role";
        const OLD_VALUE: &str = "0";
        const NEW_VALUE: &str = "101";

        fn _role_constraint() -> Constraint {
            Constraint::RoleConstraint(RoleConstraint {
                sig_count: 0,
                metadata: None,
                role: Some(String::new()),
                need_to_be_owner: false,
                off_ledger_signature: false,
            })
        }

        fn _role_constraint_json() -> String {
            serde_json::to_string(&_role_constraint()).unwrap()
        }

        #[test]
        fn build_auth_rule_request_works_for_role_constraint() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": AUTH_RULE,
                "auth_type": NYM,
                "field": FIELD,
                "new_value": NEW_VALUE,
                "auth_action": AuthAction::ADD,
                "constraint": _role_constraint(),
            });

            let request = ledger_service
                .build_auth_rule_request(
                    &identifier(),
                    NYM,
                    ADD_AUTH_ACTION,
                    FIELD,
                    None,
                    Some(NEW_VALUE),
                    _role_constraint(),
                )
                .unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_auth_rule_request_works_for_combination_constraints() {
            let ledger_service = LedgerService::new();

            let constraint = Constraint::AndConstraint(CombinationConstraint {
                auth_constraints: vec![
                    _role_constraint(),
                    Constraint::OrConstraint(CombinationConstraint {
                        auth_constraints: vec![_role_constraint(), _role_constraint()],
                    }),
                ],
            });

            let expected_result = json!({
                "type": AUTH_RULE,
                "auth_type": NYM,
                "field": FIELD,
                "new_value": NEW_VALUE,
                "auth_action": AuthAction::ADD,
                "constraint": constraint,
            });

            let request = ledger_service
                .build_auth_rule_request(
                    &identifier(),
                    NYM,
                    ADD_AUTH_ACTION,
                    FIELD,
                    None,
                    Some(NEW_VALUE),
                    constraint,
                )
                .unwrap();

            check_request(&request, expected_result);
        }

        #[test]
        fn build_auth_rule_request_works_for_edit_auth_action() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": AUTH_RULE,
                "auth_type": NYM,
                "field": FIELD,
                "old_value": OLD_VALUE,
                "new_value": NEW_VALUE,
                "auth_action": AuthAction::EDIT,
                "constraint": _role_constraint(),
            });

            let request = ledger_service
                .build_auth_rule_request(
                    &identifier(),
                    NYM,
                    EDIT_AUTH_ACTION,
                    FIELD,
                    Some(OLD_VALUE),
                    Some(NEW_VALUE),
                    _role_constraint(),
                )
                .unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_auth_rule_request_works_for_invalid_auth_action() {
            let ledger_service = LedgerService::new();

            let res = ledger_service.build_auth_rule_request(
                &identifier(),
                NYM,
                "WRONG",
                FIELD,
                None,
                Some(NEW_VALUE),
                _role_constraint(),
            );
            assert_kind!(VdrErrorKind::InvalidStructure, res);
        }

        #[test]
        fn build_get_auth_rule_request_works_for_add_action() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": GET_AUTH_RULE,
                "auth_type": NYM,
                "field": FIELD,
                "new_value": NEW_VALUE,
                "auth_action": AuthAction::ADD,
            });

            let request = ledger_service
                .build_get_auth_rule_request(
                    Some(&identifier()),
                    Some(NYM),
                    Some(ADD_AUTH_ACTION),
                    Some(FIELD),
                    None,
                    Some(NEW_VALUE),
                )
                .unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_auth_rule_request_works_for_edit_action() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": GET_AUTH_RULE,
                "auth_type": NYM,
                "field": FIELD,
                "old_value": OLD_VALUE,
                "new_value": NEW_VALUE,
                "auth_action": AuthAction::EDIT,
            });

            let request = ledger_service
                .build_get_auth_rule_request(
                    Some(&identifier()),
                    Some(NYM),
                    Some(EDIT_AUTH_ACTION),
                    Some(FIELD),
                    Some(OLD_VALUE),
                    Some(NEW_VALUE),
                )
                .unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_auth_rule_request_works_for_none_params() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": GET_AUTH_RULE,
            });

            let request = ledger_service
                .build_get_auth_rule_request(Some(&identifier()), None, None, None, None, None)
                .unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_auth_rule_request_works_for_some_fields_are_specified() {
            let ledger_service = LedgerService::new();

            let res = ledger_service.build_get_auth_rule_request(
                Some(&identifier()),
                Some(NYM),
                None,
                Some(FIELD),
                None,
                None,
            );
            assert_kind!(VdrErrorKind::InvalidStructure, res);
        }

        #[test]
        fn build_get_auth_rule_request_works_for_invalid_auth_action() {
            let ledger_service = LedgerService::new();

            let res = ledger_service.build_get_auth_rule_request(
                Some(&identifier()),
                None,
                Some("WRONG"),
                None,
                None,
                None,
            );
            assert_kind!(VdrErrorKind::InvalidStructure, res);
        }

        #[test]
        fn build_get_auth_rule_request_works_for_invalid_auth_type() {
            let ledger_service = LedgerService::new();

            let res = ledger_service.build_get_auth_rule_request(
                Some(&identifier()),
                Some("WRONG"),
                None,
                None,
                None,
                None,
            );
            assert_kind!(VdrErrorKind::InvalidStructure, res);
        }

        #[test]
        fn build_auth_rules_request_works() {
            let ledger_service = LedgerService::new();

            let mut data = AuthRules::new();
            data.push(AuthRuleData::Add(AddAuthRuleData {
                auth_type: NYM.to_string(),
                field: FIELD.to_string(),
                new_value: Some(NEW_VALUE.to_string()),
                constraint: _role_constraint(),
            }));

            data.push(AuthRuleData::Edit(EditAuthRuleData {
                auth_type: NYM.to_string(),
                field: FIELD.to_string(),
                old_value: Some(OLD_VALUE.to_string()),
                new_value: Some(NEW_VALUE.to_string()),
                constraint: _role_constraint(),
            }));

            let expected_result = json!({
                "type": AUTH_RULES,
                "rules": data.clone(),
            });

            let request = ledger_service
                .build_auth_rules_request(&identifier(), data)
                .unwrap();
            check_request(&request, expected_result);
        }
    }

    mod author_agreement {
        use super::*;

        const TEXT: &str = "indy agreement";
        const VERSION: &str = "1.0.0";

        #[test]
        fn build_txn_author_agreement_request() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": TXN_AUTHR_AGRMT,
                "text": TEXT,
                "version": VERSION
            });

            let request = ledger_service
                .build_txn_author_agreement_request(&identifier(), TEXT, VERSION)
                .unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_txn_author_agreement_request_works() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({ "type": GET_TXN_AUTHR_AGRMT });

            let request = ledger_service
                .build_get_txn_author_agreement_request(Some(&identifier()), None)
                .unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_txn_author_agreement_request_for_specific_version() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": GET_TXN_AUTHR_AGRMT,
                "version": VERSION
            });

            let data = GetTxnAuthorAgreementData {
                digest: None,
                version: Some(VERSION.to_string()),
                timestamp: None,
            };

            let request = ledger_service
                .build_get_txn_author_agreement_request(Some(&identifier()), Some(&data))
                .unwrap();
            check_request(&request, expected_result);
        }
    }

    mod acceptance_mechanism {
        use super::*;

        const LABEL: &str = "label";
        const VERSION: &str = "1.0.0";
        const CONTEXT: &str = "some context";
        const TIMESTAMP: u64 = 123456789;

        fn _aml() -> AcceptanceMechanisms {
            let mut aml: AcceptanceMechanisms = AcceptanceMechanisms::new();
            aml.0.insert(
                LABEL.to_string(),
                json!({"text": "This is description for acceptance mechanism"}),
            );
            aml
        }

        #[test]
        fn build_acceptance_mechanisms_request() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": TXN_AUTHR_AGRMT_AML,
                "aml":  _aml(),
                "version":  VERSION,
            });

            let request = ledger_service
                .build_acceptance_mechanisms_request(&identifier(), _aml(), VERSION, None)
                .unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_acceptance_mechanisms_request_with_context() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": TXN_AUTHR_AGRMT_AML,
                "aml":  _aml(),
                "version":  VERSION,
                "amlContext": CONTEXT.to_string(),
            });

            let request = ledger_service
                .build_acceptance_mechanisms_request(&identifier(), _aml(), VERSION, Some(CONTEXT))
                .unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_acceptance_mechanisms_request() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": GET_TXN_AUTHR_AGRMT_AML,
            });

            let request = ledger_service
                .build_get_acceptance_mechanisms_request(None, None, None)
                .unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_acceptance_mechanisms_request_for_timestamp() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": GET_TXN_AUTHR_AGRMT_AML,
                "timestamp": TIMESTAMP,
            });

            let request = ledger_service
                .build_get_acceptance_mechanisms_request(None, Some(TIMESTAMP), None)
                .unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_acceptance_mechanisms_request_for_version() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": GET_TXN_AUTHR_AGRMT_AML,
                "version": VERSION,
            });

            let request = ledger_service
                .build_get_acceptance_mechanisms_request(None, None, Some(VERSION))
                .unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_acceptance_mechanisms_request_for_timestamp_and_version() {
            let ledger_service = LedgerService::new();

            let res = ledger_service.build_get_acceptance_mechanisms_request(
                None,
                Some(TIMESTAMP),
                Some(VERSION),
            );
            assert_kind!(VdrErrorKind::InvalidStructure, res);
        }
    }

    #[test]
    fn datetime_to_date() {
        assert_eq!(0, LedgerService::datetime_to_date_timestamp(0));
        assert_eq!(0, LedgerService::datetime_to_date_timestamp(20));
        assert_eq!(
            1562284800,
            LedgerService::datetime_to_date_timestamp(1562367600)
        );
        assert_eq!(
            1562284800,
            LedgerService::datetime_to_date_timestamp(1562319963)
        );
        assert_eq!(
            1562284800,
            LedgerService::datetime_to_date_timestamp(1562284800)
        );
    }

    fn check_request(request: &str, expected_result: SJsonValue) {
        let request: SJsonValue = serde_json::from_str(request).unwrap();
        assert_eq!(request["operation"], expected_result);
    }
}
*/
