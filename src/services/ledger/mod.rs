use hex::FromHex;
use log_derive::logfn;
use serde::de::DeserializeOwned;
use serde_json;
use serde_json::Value;

use crate::domain::did::DidValue;
use crate::domain::ledger::attrib::{AttribOperation, GetAttribOperation};
use crate::domain::ledger::auth_rule::*;
use crate::domain::ledger::author_agreement::*;
use crate::domain::ledger::constants::{
    txn_name_to_code, ENDORSER, GET_VALIDATOR_INFO, NETWORK_MONITOR, POOL_RESTART, ROLES,
    ROLE_REMOVE, STEWARD, TRUSTEE,
};
use crate::domain::ledger::ddo::GetDdoOperation;
use crate::domain::ledger::node::{NodeOperation, NodeOperationData};
use crate::domain::ledger::nym::{
    GetNymOperation, GetNymReplyResult, GetNymResultDataV0, NymData, NymOperation,
};
use crate::domain::ledger::pool::{
    PoolConfigOperation, PoolRestartOperation, PoolUpgradeOperation, Schedule,
};
use crate::domain::ledger::request::{get_request_id, Request, TxnAuthrAgrmtAcceptanceData};
use crate::domain::ledger::response::{Message, Reply, ReplyType};
use crate::domain::ledger::txn::{GetTxnOperation, LedgerType};
use crate::domain::ledger::validator_info::GetValidatorInfoOperation;
use crate::domain::pool::ProtocolVersion;
use crate::utils::error::prelude::*;
use crate::utils::hash::{DefaultHash as Hash, TreeHash};

macro_rules! build_result {
    ($proto_ver:expr, $opt_submitter_did:expr, $operation:expr) => {{
        Request::build_request(
            get_request_id(),
            $operation,
            $opt_submitter_did,
            Some($proto_ver as usize),
        )
        .map_err(|err| LedgerError::from_msg(LedgerErrorKind::InvalidState, err))
    }};
}

pub struct LedgerService {
    protocol_version: ProtocolVersion,
}

impl LedgerService {
    pub fn new() -> LedgerService {
        LedgerService {
            protocol_version: ProtocolVersion::default(),
        }
    }

    pub fn new_with_version(protocol_version: ProtocolVersion) -> LedgerService {
        LedgerService { protocol_version }
    }

    #[logfn(Info)]
    pub fn build_nym_request(
        &self,
        identifier: &DidValue,
        dest: &DidValue,
        verkey: Option<&str>,
        alias: Option<&str>,
        role: Option<&str>,
    ) -> LedgerResult<String> {
        let role = if let Some(r) = role {
            Some(if r == ROLE_REMOVE {
                Value::Null
            } else {
                json!(match r {
                    "STEWARD" => STEWARD,
                    "TRUSTEE" => TRUSTEE,
                    "TRUST_ANCHOR" | "ENDORSER" => ENDORSER,
                    "NETWORK_MONITOR" => NETWORK_MONITOR,
                    role if ROLES.contains(&role) => role,
                    role =>
                        return Err(err_msg(
                            LedgerErrorKind::InvalidStructure,
                            format!("Invalid role: {}", role)
                        )),
                })
            })
        } else {
            None
        };

        build_result!(
            self.protocol_version,
            Some(identifier),
            NymOperation::new(
                dest.to_short(),
                verkey.map(String::from),
                alias.map(String::from),
                role
            )
        )
    }

    #[logfn(Info)]
    pub fn build_get_nym_request(
        &self,
        identifier: Option<&DidValue>,
        dest: &DidValue,
    ) -> LedgerResult<String> {
        build_result!(
            self.protocol_version,
            identifier,
            GetNymOperation::new(dest.to_short())
        )
    }

    #[logfn(Info)]
    pub fn parse_get_nym_response(&self, get_nym_response: &str) -> LedgerResult<String> {
        let reply: Reply<GetNymReplyResult> = LedgerService::parse_response(get_nym_response)?;

        let nym_data = match reply.result() {
            GetNymReplyResult::GetNymReplyResultV0(res) => {
                let data: GetNymResultDataV0 = res
                    .data
                    .ok_or(LedgerError::from_msg(
                        LedgerErrorKind::ItemNotFound,
                        format!("Nym not found"),
                    ))
                    .and_then(|data| {
                        serde_json::from_str(&data).map_err(|err| {
                            LedgerError::from_msg(
                                LedgerErrorKind::InvalidState,
                                format!("Cannot parse GET_NYM response: {}", err),
                            )
                        })
                    })?;

                NymData {
                    did: data.dest,
                    verkey: data.verkey,
                    role: data.role,
                }
            }
            GetNymReplyResult::GetNymReplyResultV1(res) => NymData {
                did: res.txn.data.did,
                verkey: res.txn.data.verkey,
                role: res.txn.data.role,
            },
        };

        let res = serde_json::to_string(&nym_data).map_err(|err| {
            LedgerError::from_msg(
                LedgerErrorKind::InvalidState,
                format!("Cannot serialize NYM data: {}", err),
            )
        })?;

        Ok(res)
    }

    #[logfn(Info)]
    pub fn build_get_ddo_request(
        &self,
        identifier: Option<&DidValue>,
        dest: &DidValue,
    ) -> LedgerResult<String> {
        build_result!(
            self.protocol_version,
            identifier,
            GetDdoOperation::new(dest.to_short())
        )
    }

    #[logfn(Info)]
    pub fn build_attrib_request(
        &self,
        identifier: &DidValue,
        dest: &DidValue,
        hash: Option<&str>,
        raw: Option<&serde_json::Value>,
        enc: Option<&str>,
    ) -> LedgerResult<String> {
        build_result!(
            self.protocol_version,
            Some(identifier),
            AttribOperation::new(
                dest.to_short(),
                hash.map(String::from),
                raw.map(serde_json::Value::to_string),
                enc.map(String::from)
            )
        )
    }

    #[logfn(Info)]
    pub fn build_get_attrib_request(
        &self,
        identifier: Option<&DidValue>,
        dest: &DidValue,
        raw: Option<&str>,
        hash: Option<&str>,
        enc: Option<&str>,
    ) -> LedgerResult<String> {
        build_result!(
            self.protocol_version,
            identifier,
            GetAttribOperation::new(dest.to_short(), raw, hash, enc)
        )
    }

    #[logfn(Info)]
    pub fn build_node_request(
        &self,
        identifier: &DidValue,
        dest: &DidValue,
        data: NodeOperationData,
    ) -> LedgerResult<String> {
        build_result!(
            self.protocol_version,
            Some(identifier),
            NodeOperation::new(dest.to_short(), data)
        )
    }

    #[logfn(Info)]
    pub fn build_get_validator_info_request(&self, identifier: &DidValue) -> LedgerResult<String> {
        build_result!(
            self.protocol_version,
            Some(identifier),
            GetValidatorInfoOperation::new()
        )
    }

    #[logfn(Info)]
    pub fn build_get_txn_request(
        &self,
        identifier: Option<&DidValue>,
        ledger_type: Option<&str>,
        seq_no: i32,
    ) -> LedgerResult<String> {
        let ledger_id = match ledger_type {
            Some(type_) => serde_json::from_str::<LedgerType>(&format!(r#""{}""#, type_))
                .map(|type_| type_.to_id())
                .or_else(|_| type_.parse::<i32>())
                .to_result(
                    LedgerErrorKind::InvalidStructure,
                    format!("Invalid Ledger type: {}", type_),
                )?,
            None => LedgerType::DOMAIN.to_id(),
        };

        build_result!(
            self.protocol_version,
            identifier,
            GetTxnOperation::new(seq_no, ledger_id)
        )
    }

    #[logfn(Info)]
    pub fn build_pool_config(
        &self,
        identifier: &DidValue,
        writes: bool,
        force: bool,
    ) -> LedgerResult<String> {
        build_result!(
            self.protocol_version,
            Some(identifier),
            PoolConfigOperation::new(writes, force)
        )
    }

    #[logfn(Info)]
    pub fn build_pool_restart(
        &self,
        identifier: &DidValue,
        action: &str,
        datetime: Option<&str>,
    ) -> LedgerResult<String> {
        build_result!(
            self.protocol_version,
            Some(identifier),
            PoolRestartOperation::new(action, datetime.map(String::from))
        )
    }

    #[logfn(Info)]
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
    ) -> LedgerResult<String> {
        build_result!(
            self.protocol_version,
            Some(identifier),
            PoolUpgradeOperation::new(
                name,
                version,
                action,
                sha256,
                timeout,
                schedule,
                justification,
                reinstall,
                force,
                package
            )
        )
    }

    #[logfn(Info)]
    pub fn build_auth_rule_request(
        &self,
        submitter_did: &DidValue,
        txn_type: &str,
        action: &str,
        field: &str,
        old_value: Option<&str>,
        new_value: Option<&str>,
        constraint: Constraint,
    ) -> LedgerResult<String> {
        let txn_type = txn_name_to_code(&txn_type).ok_or_else(|| {
            err_msg(
                LedgerErrorKind::InvalidStructure,
                format!("Unsupported `txn_type`: {}", txn_type),
            )
        })?;

        let action =
            serde_json::from_str::<AuthAction>(&format!("\"{}\"", action)).map_err(|err| {
                LedgerError::from_msg(
                    LedgerErrorKind::InvalidStructure,
                    format!("Cannot parse auth action: {}", err),
                )
            })?;

        build_result!(
            self.protocol_version,
            Some(submitter_did),
            AuthRuleOperation::new(
                txn_type.to_string(),
                field.to_string(),
                action,
                old_value.map(String::from),
                new_value.map(String::from),
                constraint
            )
        )
    }

    #[logfn(Info)]
    pub fn build_auth_rules_request(
        &self,
        submitter_did: &DidValue,
        rules: AuthRules,
    ) -> LedgerResult<String> {
        build_result!(
            self.protocol_version,
            Some(submitter_did),
            AuthRulesOperation::new(rules)
        )
    }

    #[logfn(Info)]
    pub fn build_get_auth_rule_request(
        &self,
        submitter_did: Option<&DidValue>,
        auth_type: Option<&str>,
        auth_action: Option<&str>,
        field: Option<&str>,
        old_value: Option<&str>,
        new_value: Option<&str>,
    ) -> LedgerResult<String> {
        let operation = match (auth_type, auth_action, field) {
            (None, None, None) => GetAuthRuleOperation::get_all(),
            (Some(auth_type), Some(auth_action), Some(field)) => {
                let type_ = txn_name_to_code(&auth_type).ok_or_else(|| {
                    err_msg(
                        LedgerErrorKind::InvalidStructure,
                        format!("Unsupported `auth_type`: {}", auth_type),
                    )
                })?;

                let action = serde_json::from_str::<AuthAction>(&format!("\"{}\"", auth_action))
                    .map_err(|err| {
                        LedgerError::from_msg(
                            LedgerErrorKind::InvalidStructure,
                            format!("Cannot parse auth action: {}", err),
                        )
                    })?;

                GetAuthRuleOperation::get_one(
                    type_.to_string(),
                    field.to_string(),
                    action,
                    old_value.map(String::from),
                    new_value.map(String::from),
                )
            }
            _ => {
                return Err(err_msg(
                    LedgerErrorKind::InvalidStructure,
                    "Either none or all transaction related parameters must be specified.",
                ))
            }
        };

        build_result!(self.protocol_version, submitter_did, operation)
    }

    #[logfn(Info)]
    pub fn build_txn_author_agreement_request(
        &self,
        identifier: &DidValue,
        text: &str,
        version: &str,
    ) -> LedgerResult<String> {
        build_result!(
            self.protocol_version,
            Some(identifier),
            TxnAuthorAgreementOperation::new(text.to_string(), version.to_string())
        )
    }

    #[logfn(Info)]
    pub fn build_get_txn_author_agreement_request(
        &self,
        identifier: Option<&DidValue>,
        data: Option<&GetTxnAuthorAgreementData>,
    ) -> LedgerResult<String> {
        build_result!(
            self.protocol_version,
            identifier,
            GetTxnAuthorAgreementOperation::new(data)
        )
    }

    #[logfn(Info)]
    pub fn build_acceptance_mechanisms_request(
        &self,
        identifier: &DidValue,
        aml: AcceptanceMechanisms,
        version: &str,
        aml_context: Option<&str>,
    ) -> LedgerResult<String> {
        build_result!(
            self.protocol_version,
            Some(identifier),
            SetAcceptanceMechanismOperation::new(
                aml,
                version.to_string(),
                aml_context.map(String::from)
            )
        )
    }

    #[logfn(Info)]
    pub fn build_get_acceptance_mechanisms_request(
        &self,
        identifier: Option<&DidValue>,
        timestamp: Option<u64>,
        version: Option<&str>,
    ) -> LedgerResult<String> {
        if timestamp.is_some() && version.is_some() {
            return Err(err_msg(
                LedgerErrorKind::InvalidStructure,
                "timestamp and version cannot be specified together.",
            ));
        }

        build_result!(
            self.protocol_version,
            identifier,
            GetAcceptanceMechanismOperation::new(timestamp, version.map(String::from))
        )
    }

    #[logfn(Info)]
    pub fn parse_response<T>(response: &str) -> LedgerResult<Reply<T>>
    where
        T: DeserializeOwned + ReplyType + ::std::fmt::Debug,
    {
        let message: serde_json::Value = serde_json::from_str(&response).to_result(
            LedgerErrorKind::InvalidTransaction,
            "Response is invalid json",
        )?;

        if message["op"] == json!("REPLY") && message["result"]["type"] != json!(T::get_type()) {
            return Err(err_msg(
                LedgerErrorKind::InvalidTransaction,
                "Invalid response type",
            ));
        }

        let message: Message<T> = serde_json::from_value(message).to_result(
            LedgerErrorKind::ItemNotFound,
            "Structure doesn't correspond to type. Most probably not found",
        )?; // FIXME: Review how we handle not found

        match message {
            Message::Reject(response) | Message::ReqNACK(response) => Err(err_msg(
                LedgerErrorKind::InvalidTransaction,
                format!("Transaction has been failed: {:?}", response.reason),
            )),
            Message::Reply(reply) => Ok(reply),
        }
    }

    #[logfn(Info)]
    pub fn validate_action(&self, request: &str) -> LedgerResult<()> {
        let request: Request<serde_json::Value> = serde_json::from_str(request).map_err(|err| {
            LedgerError::from_msg(
                LedgerErrorKind::InvalidStructure,
                format!("Request is invalid json: {:?}", err),
            )
        })?;

        match request.operation["type"].as_str() {
            Some(POOL_RESTART) | Some(GET_VALIDATOR_INFO) => Ok(()),
            Some(_) => Err(err_msg(
                LedgerErrorKind::InvalidStructure,
                "Request does not match any type of Actions: POOL_RESTART, GET_VALIDATOR_INFO",
            )),
            None => Err(err_msg(
                LedgerErrorKind::InvalidStructure,
                "No valid type field in request",
            )),
        }
    }

    #[logfn(Info)]
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
                return Err(err_msg(LedgerErrorKind::InvalidStructure, "Invalid combination of params: Either combination `text` + `version` or `taa_digest` must be passed."));
            }
            (None, None, Some(hash_)) => hash_.to_string(),
            (Some(_), None, _) | (None, Some(_), _) => {
                return Err(err_msg(LedgerErrorKind::InvalidStructure, "Invalid combination of params: `text` and `version` should be passed or skipped together."));
            }
            (Some(text_), Some(version_), None) => {
                hex::encode(self._calculate_hash(text_, version_)?)
            }
            (Some(text_), Some(version_), Some(hash_)) => {
                self._compare_hash(text_, version_, hash_)?;
                hash_.to_string()
            }
        };

        let acceptance_data = TxnAuthrAgrmtAcceptanceData {
            mechanism: mechanism.to_string(),
            taa_digest,
            time: LedgerService::datetime_to_date_timestamp(time),
        };

        Ok(acceptance_data)
    }

    fn datetime_to_date_timestamp(time: u64) -> u64 {
        const SEC_IN_DAY: u64 = 86400;
        time / SEC_IN_DAY * SEC_IN_DAY
    }

    fn _calculate_hash(&self, text: &str, version: &str) -> LedgerResult<Vec<u8>> {
        let content: String = version.to_string() + text;
        Hash::hash(content.as_bytes())
    }

    fn _compare_hash(&self, text: &str, version: &str, hash: &str) -> LedgerResult<()> {
        let calculated_hash = self._calculate_hash(text, version)?;

        let passed_hash = Vec::from_hex(hash).map_err(|err| {
            LedgerError::from_msg(
                LedgerErrorKind::InvalidStructure,
                format!("Cannot decode `hash`: {:?}", err),
            )
        })?;

        if calculated_hash != passed_hash {
            return Err(LedgerError::from_msg(LedgerErrorKind::InvalidStructure,
                                           format!("Calculated hash of concatenation `version` and `text` doesn't equal to passed `hash` value. \n\
                                           Calculated hash value: {:?}, \n Passed hash value: {:?}", calculated_hash, passed_hash)));
        }
        Ok(())
    }

    pub fn parse_get_auth_rule_response(&self, response: &str) -> LedgerResult<Vec<AuthRule>> {
        trace!("parse_get_auth_rule_response >>> response: {:?}", response);

        let response: Reply<GetAuthRuleResult> =
            serde_json::from_str(&response).map_err(|err| {
                LedgerError::from_msg(
                    LedgerErrorKind::InvalidTransaction,
                    format!("Cannot parse GetAuthRule response: {:?}", err),
                )
            })?;

        let res = response.result().data;

        trace!("parse_get_auth_rule_response <<< {:?}", res);

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::ledger::constants::*;
    use crate::domain::ledger::node::Services;

    use super::*;

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
            "role": serde_json::Value::Null,
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
            "role": serde_json::Value::Null,
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

    fn check_request(request: &str, expected_result: serde_json::Value) {
        let request: serde_json::Value = serde_json::from_str(request).unwrap();
        assert_eq!(request["operation"], expected_result);
    }
}
