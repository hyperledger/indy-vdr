pub mod constants;
pub mod domain;

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
use crate::utils::crypto::{import_keypair, sign_message};
use crate::utils::hash::{digest, Sha256};
use crate::utils::signature::serialize_signature;

use self::domain::attrib::{AttribOperation, GetAttribOperation};
use self::domain::auth_rule::*;
use self::domain::author_agreement::*;
use self::domain::ddo::GetDdoOperation;
use self::domain::node::{NodeOperation, NodeOperationData};
use self::domain::nym::{GetNymOperation, NymOperation};
use self::domain::pool::{
    PoolConfigOperation, PoolRestartOperation, PoolUpgradeOperation, Schedule,
};
use self::domain::request::RequestType;
use self::domain::request::{get_request_id, Request, TxnAuthrAgrmtAcceptanceData};
use self::domain::txn::GetTxnOperation;
use self::domain::validator_info::GetValidatorInfoOperation;

use self::constants::{
    txn_name_to_code, ENDORSER, NETWORK_MONITOR, ROLES, ROLE_REMOVE, STEWARD, TRUSTEE,
};

fn datetime_to_date_timestamp(time: u64) -> u64 {
    const SEC_IN_DAY: u64 = 86400;
    time / SEC_IN_DAY * SEC_IN_DAY
}

fn calculate_hash(text: &str, version: &str) -> LedgerResult<Vec<u8>> {
    let content: String = version.to_string() + text;
    Ok(digest::<Sha256>(content.as_bytes()))
}

fn compare_hash(text: &str, version: &str, hash: &str) -> LedgerResult<()> {
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
    pub txn_type: String,
    pub req_id: String,
    pub req_json: SJsonValue,
    pub sp_key: Option<Vec<u8>>,
    pub sp_timestamps: (Option<u64>, Option<u64>),
}

impl PreparedRequest {
    pub fn new(
        txn_type: String,
        req_id: String,
        req_json: SJsonValue,
        sp_key: Option<Vec<u8>>,
        sp_timestamps: (Option<u64>, Option<u64>),
    ) -> Self {
        Self {
            txn_type,
            req_id,
            req_json,
            sp_key,
            sp_timestamps,
        }
    }

    pub fn get_signature_input(&self) -> LedgerResult<String> {
        serialize_signature(&self.req_json)
    }

    pub fn sign(&mut self, secret: &[u8]) -> LedgerResult<()> {
        let keypair = import_keypair(&secret)?;
        let input = self.get_signature_input()?;
        let sig = sign_message(keypair, input.as_bytes());
        self.req_json["signature"] = SJsonValue::String(sig.to_base58());
        Ok(())
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
    ) -> LedgerResult<PreparedRequest> {
        let req_id = get_request_id();
        let identifier = identifier.or(Some(&DEFAULT_LIBINDY_DID));
        let txn_type = T::get_txn_type().to_string();
        let sp_key = operation.get_sp_key(self.protocol_version)?;
        let sp_timestamps = operation.get_sp_timestamps()?;
        let body = Request::build_request(
            req_id,
            operation,
            identifier,
            Some(self.protocol_version as usize),
        )?;
        Ok(PreparedRequest::new(
            txn_type,
            req_id.to_string(),
            body,
            sp_key,
            sp_timestamps,
        ))
    }

    pub fn build_nym_request(
        &self,
        identifier: &DidValue,
        dest: &DidValue,
        verkey: Option<String>,
        alias: Option<String>,
        role: Option<String>,
    ) -> LedgerResult<PreparedRequest> {
        let role = if let Some(r) = role {
            Some(if r == ROLE_REMOVE {
                SJsonValue::Null
            } else {
                json!(match r.as_str() {
                    "STEWARD" => STEWARD,
                    "TRUSTEE" => TRUSTEE,
                    "TRUST_ANCHOR" | "ENDORSER" => ENDORSER,
                    "NETWORK_MONITOR" => NETWORK_MONITOR,
                    role if ROLES.contains(&role) => role,
                    role => return Err(input_err(format!("Invalid role: {}", role))),
                })
            })
        } else {
            None
        };
        let operation = NymOperation::new(dest.to_short(), verkey, alias, role);
        self.build(operation, Some(identifier))
    }

    pub fn build_get_nym_request(
        &self,
        identifier: Option<&DidValue>,
        dest: &DidValue,
    ) -> LedgerResult<PreparedRequest> {
        let dest = dest.to_short();
        let operation = GetNymOperation::new(dest.clone());
        self.build(operation, identifier)
    }

    pub fn build_get_ddo_request(
        &self,
        identifier: Option<&DidValue>,
        dest: &DidValue,
    ) -> LedgerResult<PreparedRequest> {
        let operation = GetDdoOperation::new(dest.to_short());
        self.build(operation, identifier)
    }

    pub fn build_attrib_request(
        &self,
        identifier: &DidValue,
        dest: &DidValue,
        hash: Option<String>,
        raw: Option<&SJsonValue>,
        enc: Option<String>,
    ) -> LedgerResult<PreparedRequest> {
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
    ) -> LedgerResult<PreparedRequest> {
        let operation = GetAttribOperation::new(dest.to_short(), raw, hash, enc);
        self.build(operation, identifier)
    }

    pub fn build_node_request(
        &self,
        identifier: &DidValue,
        dest: &DidValue,
        data: NodeOperationData,
    ) -> LedgerResult<PreparedRequest> {
        let operation = NodeOperation::new(dest.to_short(), data);
        self.build(operation, Some(identifier))
    }

    pub fn build_get_validator_info_request(
        &self,
        identifier: &DidValue,
    ) -> LedgerResult<PreparedRequest> {
        self.build(GetValidatorInfoOperation::new(), Some(identifier))
    }

    pub fn build_get_txn_request(
        &self,
        ledger_type: i32,
        seq_no: i32,
        identifier: Option<&DidValue>,
    ) -> LedgerResult<PreparedRequest> {
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
    ) -> LedgerResult<PreparedRequest> {
        self.build(PoolConfigOperation::new(writes, force), Some(identifier))
    }

    pub fn build_pool_restart(
        &self,
        identifier: &DidValue,
        action: &str,
        datetime: Option<&str>,
    ) -> LedgerResult<PreparedRequest> {
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
    ) -> LedgerResult<PreparedRequest> {
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
    ) -> LedgerResult<PreparedRequest> {
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
    ) -> LedgerResult<PreparedRequest> {
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
    ) -> LedgerResult<PreparedRequest> {
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
                    old_value,
                    new_value,
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
        text: String,
        version: String,
    ) -> LedgerResult<PreparedRequest> {
        self.build(
            TxnAuthorAgreementOperation::new(text, version),
            Some(identifier),
        )
    }

    pub fn build_get_txn_author_agreement_request(
        &self,
        identifier: Option<&DidValue>,
        data: Option<&GetTxnAuthorAgreementData>,
    ) -> LedgerResult<PreparedRequest> {
        self.build(GetTxnAuthorAgreementOperation::new(data), identifier)
    }

    pub fn build_acceptance_mechanisms_request(
        &self,
        identifier: &DidValue,
        aml: AcceptanceMechanisms,
        version: String,
        aml_context: Option<String>,
    ) -> LedgerResult<PreparedRequest> {
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
    ) -> LedgerResult<PreparedRequest> {
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

    pub fn prepare_acceptance_data(
        &self,
        text: Option<&str>,
        version: Option<&str>,
        hash: Option<&str>,
        mechanism: &str,
        time: u64,
    ) -> LedgerResult<TxnAuthrAgrmtAcceptanceData> {
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

    pub fn parse_inbound_request(
        &self,
        message: &[u8],
    ) -> LedgerResult<(PreparedRequest, Option<RequestTarget>)> {
        let message = std::str::from_utf8(message).with_input_err("Invalid UTF-8")?;
        self.parse_inbound_request_str(message)
    }

    pub fn parse_inbound_request_str(
        &self,
        message: &str,
    ) -> LedgerResult<(PreparedRequest, Option<RequestTarget>)> {
        let req_json: SJsonValue =
            serde_json::from_str(message).with_input_err("Invalid request JSON")?;
        error!("Message: {}", req_json);

        let protocol_version = ProtocolVersion::from_id(
            req_json["protocolVersion"]
                .as_u64()
                .ok_or_else(|| input_err("No protocol version request"))?,
        )?;

        let req_id = req_json["reqId"]
            .as_u64()
            .ok_or_else(|| input_err("No reqId in request"))?
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

        let sp_key = parse_key_from_request_for_builtin_sp(&req_json, protocol_version);
        let sp_timestamps = parse_timestamp_from_req_for_builtin_sp(&req_json, txn_type.as_str());
        Ok((
            PreparedRequest::new(txn_type, req_id, req_json, sp_key, sp_timestamps),
            target,
        ))
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
        assert_kind!(LedgerErrorKind::InvalidStructure, res);
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
            assert_kind!(LedgerErrorKind::InvalidStructure, res);
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
            assert_kind!(LedgerErrorKind::InvalidStructure, res);
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
            assert_kind!(LedgerErrorKind::InvalidStructure, res);
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
            assert_kind!(LedgerErrorKind::InvalidStructure, res);
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
            assert_kind!(LedgerErrorKind::InvalidStructure, res);
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
