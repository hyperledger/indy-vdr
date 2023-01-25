use hex::FromHex;
use serde_json::{self, Value as SJsonValue};

use crate::common::error::prelude::*;
use crate::pool::{new_request_id, PreparedRequest, ProtocolVersion, RequestMethod};
use crate::utils::did::{DidValue, DEFAULT_LIBINDY_DID};
use crate::utils::hash::SHA256;
use crate::utils::Qualifiable;

#[cfg(any(feature = "rich_schema", test))]
use super::identifiers::RichSchemaId;
use super::identifiers::{CredentialDefinitionId, RevocationRegistryId, SchemaId};
use super::requests::attrib::{AttribOperation, GetAttribOperation};
use super::requests::auth_rule::{
    AuthAction, AuthRuleOperation, AuthRules, AuthRulesOperation, Constraint, GetAuthRuleOperation,
};
use super::requests::author_agreement::{
    AcceptanceMechanisms, DisableAllTxnAuthorAgreementsOperation, GetAcceptanceMechanismOperation,
    GetTxnAuthorAgreementData, GetTxnAuthorAgreementOperation, SetAcceptanceMechanismOperation,
    TxnAuthorAgreementOperation, TxnAuthrAgrmtAcceptanceData,
};
use super::requests::cred_def::{CredDefOperation, CredentialDefinition, GetCredDefOperation};
use super::requests::ledgers_freeze::{GetFrozenLedgersOperation, LedgersFreezeOperation};
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
#[cfg(any(feature = "rich_schema", test))]
use super::requests::rich_schema::{
    GetRichSchemaById, GetRichSchemaByIdOperation, GetRichSchemaByMetadata,
    GetRichSchemaByMetadataOperation, RSContent, RSContextOperation, RSCredDefOperation,
    RSEncodingOperation, RSMappingOperation, RSPresDefOperation, RSType, RichSchema,
    RichSchemaBaseOperation, RichSchemaOperation,
};
use super::requests::schema::{
    GetSchemaOperation, GetSchemaOperationData, Schema, SchemaOperation, SchemaOperationData,
};
use super::requests::txn::GetTxnOperation;
use super::requests::validator_info::GetValidatorInfoOperation;
use super::requests::{Request, RequestType};

use super::constants::txn_name_to_code;

fn datetime_to_date_timestamp(time: u64) -> u64 {
    const SEC_IN_DAY: u64 = 86400;
    time / SEC_IN_DAY * SEC_IN_DAY
}

fn calculate_hash(text: &str, version: &str) -> VdrResult<Vec<u8>> {
    let content: String = version.to_string() + text;
    Ok(SHA256::digest(content.as_bytes()))
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

/// A utility class for constructing ledger transaction requests
pub struct RequestBuilder {
    pub protocol_version: ProtocolVersion,
}

impl Default for RequestBuilder {
    fn default() -> Self {
        Self::new(ProtocolVersion::default())
    }
}

impl RequestBuilder {
    /// Create a new `RequestBuilder` for a specific protocol version
    pub fn new(protocol_version: ProtocolVersion) -> Self {
        Self { protocol_version }
    }

    /// Build a generic prepared request
    pub fn build<T: RequestType>(
        &self,
        operation: T,
        identifier: Option<&DidValue>,
    ) -> VdrResult<PreparedRequest> {
        let req_id = new_request_id();
        let identifier = identifier.or(Some(&DEFAULT_LIBINDY_DID));
        let txn_type = T::get_txn_type().to_string();
        let sp_key = operation.get_sp_key(self.protocol_version)?;
        let method = if let Some(sp_key) = sp_key {
            Some(RequestMethod::BuiltinStateProof {
                sp_key,
                sp_timestamps: operation.get_sp_timestamps()?,
            })
        } else {
            None
        };
        let body = Request::build_request(
            req_id,
            operation,
            identifier,
            Some(self.protocol_version as i64),
        )?;
        trace!("Prepared request: {} {}", req_id, body);
        Ok(PreparedRequest::new(
            self.protocol_version,
            txn_type,
            req_id.to_string(),
            body,
            method,
        ))
    }

    /// Build a `NYM` transaction request
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

    /// Build a `GET_NYM` transaction request
    pub fn build_get_nym_request(
        &self,
        identifier: Option<&DidValue>,
        dest: &DidValue,
    ) -> VdrResult<PreparedRequest> {
        let dest = dest.to_short();
        let operation = GetNymOperation::new(dest);
        self.build(operation, identifier)
    }

    /// Build an `ATTRIB` transaction request
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

    /// Build a `GET_ATTRIB` transaction request
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

    /// Build a `NODE` transaction request
    pub fn build_node_request(
        &self,
        identifier: &DidValue,
        dest: &DidValue,
        data: NodeOperationData,
    ) -> VdrResult<PreparedRequest> {
        let operation = NodeOperation::new(dest.to_short(), data);
        self.build(operation, Some(identifier))
    }

    /// Build a `GET_VALIDATOR_INFO` transaction request
    pub fn build_get_validator_info_request(
        &self,
        identifier: &DidValue,
    ) -> VdrResult<PreparedRequest> {
        self.build(GetValidatorInfoOperation::new(), Some(identifier))
    }

    /// Build a `GET_TXN` transaction request
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

    /// Build a `POOL_CONFIG` transaction request
    pub fn build_pool_config_request(
        &self,
        identifier: &DidValue,
        writes: bool,
        force: bool,
    ) -> VdrResult<PreparedRequest> {
        self.build(PoolConfigOperation::new(writes, force), Some(identifier))
    }

    /// Build a `POOL_RESTART` transaction request
    pub fn build_pool_restart_request(
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

    /// Build a `POOL_UPGRADE` transaction request
    #[allow(clippy::too_many_arguments)]
    pub fn build_pool_upgrade_request(
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

    /// Build an `AUTH_RULE` transaction request
    #[allow(clippy::too_many_arguments)]
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

    /// Build an `AUTH_RULES` transaction request
    pub fn build_auth_rules_request(
        &self,
        submitter_did: &DidValue,
        rules: AuthRules,
    ) -> VdrResult<PreparedRequest> {
        self.build(AuthRulesOperation::new(rules), Some(submitter_did))
    }

    /// Build a `GET_AUTH_RULE` transaction request
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
                ));
            }
        };
        self.build(operation, submitter_did)
    }

    /// Build a `TXN_AUTHR_AGRMT` transacation request
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

    /// Build a `GET_TXN_AUTHR_AGRMT` transaction request
    pub fn build_get_txn_author_agreement_request(
        &self,
        identifier: Option<&DidValue>,
        data: Option<&GetTxnAuthorAgreementData>,
    ) -> VdrResult<PreparedRequest> {
        self.build(GetTxnAuthorAgreementOperation::new(data), identifier)
    }

    /// Build a `DISABLE_ALL_TXN_AUTHR_AGRMTS` transaction request
    pub fn build_disable_all_txn_author_agreements_request(
        &self,
        identifier: &DidValue,
    ) -> VdrResult<PreparedRequest> {
        self.build(
            DisableAllTxnAuthorAgreementsOperation::new(),
            Some(identifier),
        )
    }

    /// Build a `TXN_AUTHR_AGRMT_AML` transaction request
    pub fn build_acceptance_mechanisms_request(
        &self,
        identifier: &DidValue,
        aml: AcceptanceMechanisms,
        version: String,
        aml_context: Option<String>,
    ) -> VdrResult<PreparedRequest> {
        let operation =
            SetAcceptanceMechanismOperation::new(aml, version, aml_context.map(String::from));
        self.build(operation, Some(identifier))
    }

    /// Build a `GET_TXN_AUTHR_AGRMT_AML` transaction request
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

    /// Build a `SCHEMA` transaction request
    pub fn build_schema_request(
        &self,
        identifier: &DidValue,
        schema: Schema,
    ) -> VdrResult<PreparedRequest> {
        let Schema::SchemaV1(schema) = schema;
        let schema_data =
            SchemaOperationData::new(schema.name, schema.version, schema.attr_names.into());
        self.build(SchemaOperation::new(schema_data), Some(identifier))
    }

    /// Build a `GET_SCHEMA` transaction request
    pub fn build_get_schema_request(
        &self,
        identifier: Option<&DidValue>,
        id: &SchemaId,
    ) -> VdrResult<PreparedRequest> {
        let id = id.to_unqualified();
        let (_, dest, name, version) = id.parts().ok_or_else(|| {
            input_err(format!(
                "Schema ID `{}` cannot be used to build request: invalid number of parts",
                id.0
            ))
        })?;
        let data = GetSchemaOperationData::new(name, version);
        self.build(GetSchemaOperation::new(dest.to_short(), data), identifier)
    }

    /// Build a `CRED_DEF` transaction request
    pub fn build_cred_def_request(
        &self,
        identifier: &DidValue,
        cred_def: CredentialDefinition,
    ) -> VdrResult<PreparedRequest> {
        let CredentialDefinition::CredentialDefinitionV1(cred_def) = cred_def;
        self.build(CredDefOperation::new(cred_def), Some(identifier))
    }

    /// Build a `GET_CRED_DEF` transaction request
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

    /// Build a `GET_REVOC_REG_DEF` transaction request
    pub fn build_get_revoc_reg_def_request(
        &self,
        identifier: Option<&DidValue>,
        id: &RevocationRegistryId,
    ) -> VdrResult<PreparedRequest> {
        let id = id.to_unqualified();
        self.build(GetRevRegDefOperation::new(&id), identifier)
    }

    /// Build a `GET_REVOC_REG` transaction request
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

    /// Build a `GET_REVOC_REG_DELTA` transaction request
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

    /// Build a `REVOC_REG_DEF` transaction request
    pub fn build_revoc_reg_def_request(
        &self,
        identifier: &DidValue,
        revoc_reg: RevocationRegistryDefinition,
    ) -> VdrResult<PreparedRequest> {
        let RevocationRegistryDefinition::RevocationRegistryDefinitionV1(revoc_reg) = revoc_reg;
        self.build(RevRegDefOperation::new(revoc_reg), Some(identifier))
    }

    /// Build a `REVOC_REG_ENTRY` transaction request
    pub fn build_revoc_reg_entry_request(
        &self,
        identifier: &DidValue,
        revoc_reg_def_id: &RevocationRegistryId,
        revoc_def_type: &RegistryType,
        rev_reg_entry: RevocationRegistryDelta,
    ) -> VdrResult<PreparedRequest> {
        let revoc_reg_def_id = revoc_reg_def_id.to_unqualified();
        let RevocationRegistryDelta::RevocationRegistryDeltaV1(rev_reg_entry) = rev_reg_entry;
        self.build(
            RevRegEntryOperation::new(revoc_def_type, &revoc_reg_def_id, rev_reg_entry),
            Some(identifier),
        )
    }

    /// Prepare transaction author agreement acceptance data
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

    #[cfg(any(feature = "rich_schema", test))]
    #[allow(clippy::too_many_arguments)]
    /// Build a `RICH_SCHEMA` transaction request
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
        let allowed_rs_type: RSType =
            serde_json::from_value(serde_json::value::Value::String(rs_type)).map_err(input_err)?;
        match &allowed_rs_type {
            RSType::Sch => build_rs_operation!(self, RichSchemaOperation, identifier, rich_schema),
            RSType::Map => build_rs_operation!(self, RSMappingOperation, identifier, rich_schema),
            RSType::Enc => build_rs_operation!(self, RSEncodingOperation, identifier, rich_schema),
            RSType::Ctx => build_rs_operation!(self, RSContextOperation, identifier, rich_schema),
            RSType::Cdf => build_rs_operation!(self, RSCredDefOperation, identifier, rich_schema),
            RSType::Pdf => build_rs_operation!(self, RSPresDefOperation, identifier, rich_schema),
        }
    }

    #[cfg(any(feature = "rich_schema", test))]
    /// Build a `GET_RICH_SCHEMA_BY_ID` transaction request
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

    #[cfg(any(feature = "rich_schema", test))]
    /// Build a `GET_RICH_SCHEMA_BY_METADATA` transaction request
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

    /// Build a `LEDGERS_FREEZE` transaction request
    pub fn build_ledgers_freeze_request(
        &self,
        identifier: &DidValue,
        ledgers_ids: &[u64],
    ) -> VdrResult<PreparedRequest> {
        self.build(
            LedgersFreezeOperation::new(ledgers_ids.to_vec()),
            Some(identifier),
        )
    }

    /// Build a `GET_FROZEN_LEDGERS` transaction request
    pub fn build_get_frozen_ledgers_request(
        &self,
        identifier: &DidValue,
    ) -> VdrResult<PreparedRequest> {
        self.build(GetFrozenLedgersOperation::new(), Some(identifier))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::constants;

    const REQ_ID: u64 = 1585221529670242337;
    const TYPE: &str = constants::NYM;

    fn _identifier() -> DidValue {
        DidValue(String::from("V4SGRU86Z58d6TV7PBUe6f"))
    }

    fn _dest() -> DidValue {
        DidValue(String::from("VsKV7grR1BUE29mG2Fm2kX"))
    }

    fn _protocol_version() -> i64 {
        ProtocolVersion::Node1_4.to_id()
    }

    #[fixture]
    fn request() -> serde_json::Value {
        json!({
            "identifier": _identifier(),
            "operation":{
                "dest": _dest(),
                "type": TYPE
            },
            "protocolVersion": _protocol_version(),
            "reqId": REQ_ID
        })
    }

    #[fixture]
    fn request_json(request: serde_json::Value) -> String {
        request.to_string()
    }

    #[fixture]
    fn prepared_request(request_json: String) -> PreparedRequest {
        PreparedRequest::from_request_json(&request_json).unwrap()
    }

    #[fixture]
    fn request_builder() -> RequestBuilder {
        RequestBuilder::new(ProtocolVersion::Node1_4)
    }

    #[test]
    fn empty() {
        // Empty test to run module
    }

    mod prepared_request_from_request_json {
        use super::*;

        #[rstest]
        fn test_prepared_request_from_request_json(request_json: String) {
            let request = PreparedRequest::from_request_json(&request_json).unwrap();
            assert_eq!(request.protocol_version, _protocol_version());
            assert_eq!(request.txn_type, TYPE);
            assert_eq!(request.req_id, REQ_ID.to_string());
            assert_eq!(request.method, RequestMethod::Consensus);
        }

        #[rstest]
        fn test_prepared_request_from_request_json_for_not_json() {
            let _err = PreparedRequest::from_request_json("request").unwrap_err();
        }

        #[rstest(field, case("protocolVersion"), case("reqId"), case("operation"))]
        fn test_prepared_request_from_request_json_for_missed_field(
            field: &str,
            mut request: serde_json::Value,
        ) {
            request[field] = serde_json::Value::Null;
            let _err = PreparedRequest::from_request_json(&request.to_string()).unwrap_err();
        }

        #[rstest]
        fn test_prepared_request_from_request_json_for_write_request(
            request_builder: RequestBuilder,
        ) {
            let request = request_builder
                .build_nym_request(&_identifier(), &_dest(), None, None, None)
                .unwrap();

            assert_eq!(request.txn_type, constants::NYM);
            assert_eq!(request.method, RequestMethod::Consensus);
        }

        #[rstest]
        fn test_prepared_request_from_request_json_for_get_request(
            request_builder: RequestBuilder,
        ) {
            let request = request_builder
                .build_get_nym_request(None, &_dest())
                .unwrap();

            assert_eq!(request.txn_type, constants::GET_NYM);
            assert_eq!(
                request.method,
                RequestMethod::BuiltinStateProof {
                    sp_key: vec![
                        227, 51, 254, 11, 192, 83, 219, 131, 59, 204, 0, 126, 41, 96, 118, 238,
                        152, 250, 160, 191, 198, 247, 4, 130, 44, 199, 140, 143, 18, 182, 93, 229
                    ],
                    sp_timestamps: (None, None)
                }
            );
        }

        #[rstest]
        fn test_prepared_request_from_request_json_for_get_request_with_single_timestamp(
            request_builder: RequestBuilder,
        ) {
            let timestamp = 123456789;

            let data = GetTxnAuthorAgreementData {
                digest: None,
                version: None,
                timestamp: Some(timestamp),
            };

            let request = request_builder
                .build_get_txn_author_agreement_request(None, Some(&data))
                .unwrap();

            assert_eq!(request.txn_type, constants::GET_TXN_AUTHR_AGRMT);
            assert_eq!(
                request.method,
                RequestMethod::BuiltinStateProof {
                    sp_key: vec![50, 58, 108, 97, 116, 101, 115, 116],
                    sp_timestamps: (None, Some(123456789))
                }
            );
        }

        #[rstest]
        fn test_prepared_request_from_request_json_for_get_request_with_two_timestamps(
            request_builder: RequestBuilder,
        ) {
            let rev_reg_id = RevocationRegistryId("NcYxiDXkpYi6ov5FcYDi1e:4:NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag:CL_ACCUM:TAG_1".to_string());
            let from = 123456789;
            let to = 987654321;

            let request = request_builder
                .build_get_revoc_reg_delta_request(None, &rev_reg_id, Some(from), to)
                .unwrap();

            assert_eq!(request.txn_type, constants::GET_REVOC_REG_DELTA);
            assert_eq!(
                request.method,
                RequestMethod::BuiltinStateProof {
                    sp_key: vec![
                        54, 58, 78, 99, 89, 120, 105, 68, 88, 107, 112, 89, 105, 54, 111, 118, 53,
                        70, 99, 89, 68, 105, 49, 101, 58, 52, 58, 78, 99, 89, 120, 105, 68, 88,
                        107, 112, 89, 105, 54, 111, 118, 53, 70, 99, 89, 68, 105, 49, 101, 58, 51,
                        58, 67, 76, 58, 78, 99, 89, 120, 105, 68, 88, 107, 112, 89, 105, 54, 111,
                        118, 53, 70, 99, 89, 68, 105, 49, 101, 58, 50, 58, 103, 118, 116, 58, 49,
                        46, 48, 58, 116, 97, 103, 58, 67, 76, 95, 65, 67, 67, 85, 77, 58, 84, 65,
                        71, 95, 49
                    ],
                    sp_timestamps: (Some(from as u64), Some(to as u64))
                }
            );
        }
    }

    #[rstest]
    fn test_prepared_request_get_signature_input(prepared_request: PreparedRequest) {
        let expected = String::from("identifier:V4SGRU86Z58d6TV7PBUe6f|operation:dest:VsKV7grR1BUE29mG2Fm2kX|type:1|protocolVersion:2|reqId:1585221529670242337");
        assert_eq!(expected, prepared_request.get_signature_input().unwrap());
    }

    #[rstest]
    fn test_prepared_request_set_endorser(mut prepared_request: PreparedRequest) {
        let endorser = DidValue(String::from("2PRyVHmkXQnQzJQKxHxnXC"));
        prepared_request.set_endorser(&endorser).unwrap();
        assert_eq!(json!(endorser), prepared_request.req_json["endorser"]);
    }

    fn _signature_1() -> Vec<u8> {
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9]
    }

    fn _signature_1_base58() -> String {
        String::from("kA3B2yGe2z4")
    }

    fn _signature_2() -> Vec<u8> {
        vec![9, 8, 7, 6, 5, 4, 3, 2, 1]
    }

    fn _signature_2_base58() -> String {
        String::from("7fiZcYFQEKEG")
    }

    #[rstest]
    fn test_prepared_request_set_signature(mut prepared_request: PreparedRequest) {
        prepared_request.set_signature(&_signature_1()).unwrap();
        assert_eq!(
            json!(_signature_1_base58()),
            prepared_request.req_json["signature"]
        );
    }

    #[rstest]
    fn test_prepared_request_set_multi_signature(mut prepared_request: PreparedRequest) {
        prepared_request
            .set_multi_signature(&_identifier(), &_signature_1())
            .unwrap();
        prepared_request
            .set_multi_signature(&_dest(), &_signature_2())
            .unwrap();

        assert_eq!(
            prepared_request.req_json["signatures"]
                .as_object()
                .unwrap()
                .len(),
            2
        );

        assert_eq!(
            json!(_signature_1_base58()),
            prepared_request.req_json["signatures"][_identifier().0]
        );

        assert_eq!(
            json!(_signature_2_base58()),
            prepared_request.req_json["signatures"][_dest().0]
        );
    }

    #[rstest]
    fn test_prepared_request_set_multi_signature_for_replace_signature(
        mut prepared_request: PreparedRequest,
    ) {
        prepared_request.set_signature(&_signature_1()).unwrap();
        prepared_request
            .set_multi_signature(&_dest(), &_signature_2())
            .unwrap();

        assert_eq!(
            prepared_request.req_json["signatures"]
                .as_object()
                .unwrap()
                .len(),
            2
        );

        assert_eq!(
            json!(_signature_1_base58()),
            prepared_request.req_json["signatures"][_identifier().0]
        );

        assert_eq!(
            json!(_signature_2_base58()),
            prepared_request.req_json["signatures"][_dest().0]
        );
    }

    #[rstest]
    fn test_prepared_request_set_multi_signature_twice(mut prepared_request: PreparedRequest) {
        prepared_request
            .set_multi_signature(&_identifier(), &_signature_1())
            .unwrap();
        prepared_request
            .set_multi_signature(&_identifier(), &_signature_1())
            .unwrap();

        assert_eq!(
            prepared_request.req_json["signatures"]
                .as_object()
                .unwrap()
                .len(),
            1
        );

        assert_eq!(
            json!(_signature_1_base58()),
            prepared_request.req_json["signatures"][_identifier().0]
        );
    }

    #[rstest]
    fn test_prepared_request_set_taa(mut prepared_request: PreparedRequest) {
        let taa = TxnAuthrAgrmtAcceptanceData {
            mechanism: "on_click".to_string(),
            taa_digest: "afsrw".to_string(),
            time: 123456789,
        };
        prepared_request
            .set_txn_author_agreement_acceptance(&taa)
            .unwrap();
        assert_eq!(json!(taa), prepared_request.req_json["taaAcceptance"]);
    }

    #[rstest(
        protocol_version,
        case(ProtocolVersion::Node1_3),
        case(ProtocolVersion::Node1_4)
    )]
    fn test_prepare_request_for_different_protocol_versions(protocol_version: ProtocolVersion) {
        let request = RequestBuilder::new(protocol_version)
            .build_get_nym_request(None, &_dest())
            .unwrap();

        assert_eq!(request.protocol_version, protocol_version);
        assert_eq!(
            request.req_json["protocolVersion"],
            json!(protocol_version.to_id())
        );
    }

    #[test]
    fn test_datetime_to_date() {
        assert_eq!(0, datetime_to_date_timestamp(0));
        assert_eq!(0, datetime_to_date_timestamp(20));
        assert_eq!(1562284800, datetime_to_date_timestamp(1562367600));
        assert_eq!(1562284800, datetime_to_date_timestamp(1562319963));
        assert_eq!(1562284800, datetime_to_date_timestamp(1562284800));
    }

    const TEXT: &str = "text";
    const VERSION: &str = "1.0";

    fn _hash() -> Vec<u8> {
        vec![
            57, 43, 28, 219, 43, 14, 87, 200, 231, 138, 158, 55, 154, 250, 216, 45, 207, 31, 131,
            184, 182, 90, 226, 96, 73, 111, 40, 238, 105, 118, 190, 43,
        ]
    }

    #[test]
    fn test_calculate_hash() {
        assert_eq!(_hash(), calculate_hash(TEXT, VERSION).unwrap())
    }

    #[test]
    fn test_compare_hash() {
        let hash = hex::encode(_hash());
        compare_hash(TEXT, VERSION, &hash).unwrap();
    }

    #[test]
    fn test_compare_hash_for_different_hash() {
        let hash = hex::encode(vec![1, 2, 3]);
        compare_hash(TEXT, VERSION, &hash).unwrap_err();
    }

    #[test]
    fn test_compare_hash_for_invalid_hash() {
        compare_hash(TEXT, VERSION, "hash").unwrap_err();
    }
}
