#[macro_use]
mod utils;

inject_dependencies!();

use indy_vdr::common::did::DidValue;
use indy_vdr::ledger::constants;
use indy_vdr::ledger::requests::auth_rule::{CombinationConstraint, Constraint, RoleConstraint};

use crate::utils::fixtures::*;
use crate::utils::helpers;

const TXN_TYPE: &'static str = constants::NYM;
const ADD_ACTION: &'static str = "ADD";
const EDIT_ACTION: &'static str = "EDIT";
const FIELD: &'static str = "role";
const VALUE: &'static str = "0";

fn _role_constraint() -> Constraint {
    Constraint::RoleConstraint(RoleConstraint {
        sig_count: 1,
        role: Some(constants::TRUSTEE.to_string()),
        metadata: Some(json!({})),
        need_to_be_owner: false,
        off_ledger_signature: false,
    })
}

fn _complex_constraint() -> Constraint {
    Constraint::AndConstraint(CombinationConstraint {
        auth_constraints: vec![
            _role_constraint(),
            Constraint::OrConstraint(CombinationConstraint {
                auth_constraints: vec![
                    _role_constraint(),
                    Constraint::RoleConstraint(RoleConstraint {
                        sig_count: 2,
                        role: Some("2".to_string()),
                        metadata: None,
                        need_to_be_owner: true,
                        off_ledger_signature: false,
                    }),
                ],
            }),
        ],
    })
}

#[test]
fn empty() {
    // Empty test to run module
}

#[cfg(test)]
mod builder {
    use super::*;
    use indy_vdr::ledger::RequestBuilder;

    mod auth_rule {
        use super::*;

        #[rstest]
        fn test_build_auth_rule_request_works_for_adding_new_trustee(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_auth_rule_request(
                    &trustee_did,
                    TXN_TYPE.to_string(),
                    ADD_ACTION.to_string(),
                    FIELD.to_string(),
                    None,
                    Some(VALUE.to_string()),
                    _role_constraint(),
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::AUTH_RULE,
                "auth_type": TXN_TYPE,
                "auth_action": ADD_ACTION,
                "field": FIELD,
                "new_value": VALUE,
                "constraint": json!({
                    "sig_count": 1,
                    "metadata": {},
                    "role": "0",
                    "constraint_id": "ROLE",
                    "need_to_be_owner": false
                }),
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_auth_rule_request_works_for_adding_new_identity_owner(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_auth_rule_request(
                    &trustee_did,
                    TXN_TYPE.to_string(),
                    ADD_ACTION.to_string(),
                    FIELD.to_string(),
                    None,
                    None,
                    _role_constraint(),
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::AUTH_RULE,
                "auth_type": TXN_TYPE,
                "auth_action": ADD_ACTION,
                "field": FIELD,
                "new_value": serde_json::Value::Null,
                "constraint": json!({
                    "sig_count": 1,
                    "metadata": {},
                    "role": "0",
                    "constraint_id": "ROLE",
                    "need_to_be_owner": false
                }),
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_auth_rule_request_works_for_demote_trustee(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_auth_rule_request(
                    &trustee_did,
                    TXN_TYPE.to_string(),
                    EDIT_ACTION.to_string(),
                    FIELD.to_string(),
                    Some(VALUE.to_string()),
                    None,
                    _role_constraint(),
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::AUTH_RULE,
                "auth_type": TXN_TYPE,
                "auth_action": EDIT_ACTION,
                "field": FIELD,
                "old_value": VALUE,
                "new_value": serde_json::Value::Null,
                "constraint": _role_constraint(),
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_auth_rule_request_works_for_promote_role_to_trustee(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_auth_rule_request(
                    &trustee_did,
                    TXN_TYPE.to_string(),
                    EDIT_ACTION.to_string(),
                    FIELD.to_string(),
                    None,
                    Some(VALUE.to_string()),
                    _role_constraint(),
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::AUTH_RULE,
                "auth_type": TXN_TYPE,
                "auth_action": EDIT_ACTION,
                "field": FIELD,
                "old_value": serde_json::Value::Null,
                "new_value": VALUE,
                "constraint": json!({
                    "sig_count": 1,
                    "metadata": {},
                    "role": "0",
                    "constraint_id": "ROLE",
                    "need_to_be_owner": false
                }),
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_auth_rule_request_works_for_change_trustee_to_steward(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_auth_rule_request(
                    &trustee_did,
                    TXN_TYPE.to_string(),
                    EDIT_ACTION.to_string(),
                    FIELD.to_string(),
                    Some(String::from("0")),
                    Some(String::from("2")),
                    _role_constraint(),
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::AUTH_RULE,
                "auth_type": TXN_TYPE,
                "auth_action": EDIT_ACTION,
                "field": FIELD,
                "old_value": "0",
                "new_value": "2",
                "constraint": json!({
                    "sig_count": 1,
                    "metadata": {},
                    "role": "0",
                    "constraint_id": "ROLE",
                    "need_to_be_owner": false
                }),
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_auth_rule_request_works_for_complex_constraint(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_auth_rule_request(
                    &trustee_did,
                    TXN_TYPE.to_string(),
                    ADD_ACTION.to_string(),
                    FIELD.to_string(),
                    None,
                    Some(VALUE.to_string()),
                    _complex_constraint(),
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::AUTH_RULE,
                "auth_type": TXN_TYPE,
                "auth_action": ADD_ACTION,
                "field": FIELD,
                "new_value": VALUE,
                "constraint": _complex_constraint(),
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_auth_rule_request_works_for_any_type(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let txn_type = String::from("1000000000001");

            let request = request_builder
                .build_auth_rule_request(
                    &trustee_did,
                    txn_type.clone(),
                    ADD_ACTION.to_string(),
                    FIELD.to_string(),
                    None,
                    None,
                    _role_constraint(),
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::AUTH_RULE,
                "auth_type": txn_type,
                "auth_action": ADD_ACTION,
                "field": FIELD,
                "new_value": serde_json::Value::Null,
                "constraint": _role_constraint(),
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }

    mod get_auth_rule {
        use super::*;

        #[rstest]
        fn test_build_get_auth_rule_request_works_for_adding_new_trustee(
            request_builder: RequestBuilder,
        ) {
            let request = request_builder
                .build_get_auth_rule_request(
                    None,
                    Some(TXN_TYPE.to_string()),
                    Some(ADD_ACTION.to_string()),
                    Some(FIELD.to_string()),
                    None,
                    Some(VALUE.to_string()),
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_AUTH_RULE,
                "auth_type": TXN_TYPE,
                "auth_action": ADD_ACTION,
                "field": FIELD,
                "new_value": VALUE,
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_get_auth_rule_request_works_for_adding_new_identity_owner(
            request_builder: RequestBuilder,
        ) {
            let request = request_builder
                .build_get_auth_rule_request(
                    None,
                    Some(TXN_TYPE.to_string()),
                    Some(ADD_ACTION.to_string()),
                    Some(FIELD.to_string()),
                    None,
                    None,
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_AUTH_RULE,
                "auth_type": TXN_TYPE,
                "auth_action": ADD_ACTION,
                "field": FIELD,
                "new_value": serde_json::Value::Null,
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_get_auth_rule_request_works_for_demote_trustee(
            request_builder: RequestBuilder,
        ) {
            let request = request_builder
                .build_get_auth_rule_request(
                    None,
                    Some(TXN_TYPE.to_string()),
                    Some(EDIT_ACTION.to_string()),
                    Some(FIELD.to_string()),
                    Some(VALUE.to_string()),
                    None,
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_AUTH_RULE,
                "auth_type": TXN_TYPE,
                "auth_action": EDIT_ACTION,
                "field": FIELD,
                "old_value": VALUE,
                "new_value": serde_json::Value::Null,
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_get_auth_rule_request_works_for_promote_role_to_trustee(
            request_builder: RequestBuilder,
        ) {
            let request = request_builder
                .build_get_auth_rule_request(
                    None,
                    Some(TXN_TYPE.to_string()),
                    Some(EDIT_ACTION.to_string()),
                    Some(FIELD.to_string()),
                    None,
                    Some(VALUE.to_string()),
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_AUTH_RULE,
                "auth_type": TXN_TYPE,
                "auth_action": EDIT_ACTION,
                "field": FIELD,
                "old_value": serde_json::Value::Null,
                "new_value": VALUE,
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_get_auth_rule_request_works_for_change_trustee_to_steward(
            request_builder: RequestBuilder,
        ) {
            let request = request_builder
                .build_get_auth_rule_request(
                    None,
                    Some(TXN_TYPE.to_string()),
                    Some(EDIT_ACTION.to_string()),
                    Some(FIELD.to_string()),
                    Some(String::from("0")),
                    Some(String::from("2")),
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_AUTH_RULE,
                "auth_type": TXN_TYPE,
                "auth_action": EDIT_ACTION,
                "field": FIELD,
                "old_value": "0",
                "new_value": "2",
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_get_auth_rule_request_works_for_any_type(request_builder: RequestBuilder) {
            let txn_type = String::from("1000000000001");

            let request = request_builder
                .build_get_auth_rule_request(
                    None,
                    Some(txn_type.clone()),
                    Some(ADD_ACTION.to_string()),
                    Some(FIELD.to_string()),
                    None,
                    None,
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_AUTH_RULE,
                "auth_type": txn_type,
                "auth_action": ADD_ACTION,
                "field": FIELD,
                "new_value": serde_json::Value::Null,
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_auth_rule_request_works_for_get_all(request_builder: RequestBuilder) {
            let request = request_builder
                .build_get_auth_rule_request(None, None, None, None, None, None)
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_AUTH_RULE,
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_auth_rule_request_works_for_some_fields_not_specified(
            request_builder: RequestBuilder,
        ) {
            let _err = request_builder
                .build_get_auth_rule_request(
                    None,
                    Some(TXN_TYPE.to_string()),
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap_err();
        }
    }

    mod aut_rules {
        use super::*;
        use indy_vdr::ledger::requests::auth_rule::{
            AddAuthRuleData, AuthRuleData, EditAuthRuleData,
        };

        #[rstest]
        fn test_build_auth_rules(request_builder: RequestBuilder, trustee_did: DidValue) {
            let auth_rules = vec![
                AuthRuleData::Add(AddAuthRuleData {
                    auth_type: TXN_TYPE.to_string(),
                    field: FIELD.to_string(),
                    new_value: Some(VALUE.to_string()),
                    constraint: _role_constraint(),
                }),
                AuthRuleData::Edit(EditAuthRuleData {
                    auth_type: TXN_TYPE.to_string(),
                    field: FIELD.to_string(),
                    old_value: None,
                    new_value: Some(VALUE.to_string()),
                    constraint: _role_constraint(),
                }),
            ];

            let request = request_builder
                .build_auth_rules_request(&trustee_did, auth_rules.clone())
                .unwrap();

            let expected_operation = json!({
                "type": constants::AUTH_RULES,
                "rules": auth_rules
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }
}
