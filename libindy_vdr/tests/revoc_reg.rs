#[macro_use]
mod utils;

inject_dependencies!();

use indy_vdr::common::did::DidValue;
use indy_vdr::ledger::constants;

use crate::utils::helpers;
use crate::utils::fixtures::*;
use indy_vdr::ledger::identifiers::cred_def::CredentialDefinitionId;
use indy_vdr::ledger::identifiers::rev_reg::RevocationRegistryId;
use indy_vdr::ledger::requests::rev_reg_def::{RevocationRegistryDefinition, RevocationRegistryDefinitionV1, RegistryType, RevocationRegistryDefinitionValue, IssuanceType};

const TAG: &'static str = "tag";
const REVOC_DEF_TYPE: RegistryType = RegistryType::CL_ACCUM;
const FROM: i64 = 123456789;
const TO: i64 = 987654321;

fn _cred_def_id() -> CredentialDefinitionId {
    CredentialDefinitionId("NcYxiDXkpYi6ov5FcYDi1e:3:CL:1:tag".to_string())
}

fn _rev_reg_id() -> RevocationRegistryId {
    RevocationRegistryId("NcYxiDXkpYi6ov5FcYDi1e:4:NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag:CL_ACCUM:TAG_1".to_string())
}

#[test]
fn empty() {
    // Empty test to run module
}

#[cfg(test)]
mod builder {
    use super::*;
    use indy_vdr::ledger::RequestBuilder;

    mod revoc_reg_def {
        use super::*;

        fn _public_keys() -> serde_json::Value {
            json!({
                "accumKey": {
                    "z": "1 0000000000000000000000000000000000000000000000000000000000001111 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000"
                }
            })
        }

        fn _value() -> RevocationRegistryDefinitionValue {
            RevocationRegistryDefinitionValue {
                issuance_type: IssuanceType::ISSUANCE_BY_DEFAULT,
                max_cred_num: 5,
                public_keys: serde_json::from_value(_public_keys()).unwrap(),
                tails_hash: String::from("hash"),
                tails_location: String::from("path/to/tails"),
            }
        }

        fn _rev_reg_def() -> RevocationRegistryDefinition {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(RevocationRegistryDefinitionV1 {
                id: _rev_reg_id(),
                revoc_def_type: REVOC_DEF_TYPE,
                tag: TAG.to_string(),
                cred_def_id: _cred_def_id(),
                value: _value(),
            })
        }

        #[rstest]
        fn test_build_revoc_reg_def_request(request_builder: RequestBuilder,
                                            trustee_did: DidValue) {
            let request =
                request_builder
                    .build_revoc_reg_def_request(&trustee_did,
                                                 _rev_reg_def()).unwrap();

            let expected_operation = json!({
                "id": _rev_reg_id(),
                "credDefId": _cred_def_id(),
                "revocDefType": REVOC_DEF_TYPE,
                "tag": TAG,
                "type": constants::REVOC_REG_DEF,
                "value": _value()
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }

    mod get_revoc_reg_def {
        use super::*;

        #[rstest]
        fn test_build_get_revoc_reg_def_request(request_builder: RequestBuilder) {
            let request =
                request_builder
                    .build_get_revoc_reg_def_request(None,
                                                     &_rev_reg_id()).unwrap();

            let expected_operation = json!({
                "type": constants::GET_REVOC_REG_DEF,
                "id": _rev_reg_id()
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }

    mod revoc_reg_entry {
        use super::*;
        use indy_vdr::ledger::requests::rev_reg::{RevocationRegistryDelta, RevocationRegistryDeltaV1};

        fn _value() -> serde_json::Value {
            json!({
                "accum": "1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000"
            })
        }

        fn _delta() -> RevocationRegistryDelta {
            RevocationRegistryDelta::RevocationRegistryDeltaV1(RevocationRegistryDeltaV1 {
                value: serde_json::from_value(_value()).unwrap(),
            })
        }


        #[rstest]
        fn test_build_revoc_reg_entry_request(request_builder: RequestBuilder,
                                              trustee_did: DidValue) {
            let request =
                request_builder
                    .build_revoc_reg_entry_request(&trustee_did,
                                                   &_rev_reg_id(),
                                                   &REVOC_DEF_TYPE,
                                                   _delta()).unwrap();

            let expected_operation = json!({
                "type": constants::REVOC_REG_ENTRY,
                "revocRegDefId": _rev_reg_id(),
                "revocDefType": REVOC_DEF_TYPE,
                "value": _value()
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }

    mod get_revoc_reg {
        use super::*;

        #[rstest]
        fn test_build_get_revoc_reg_delta_request(request_builder: RequestBuilder) {
            let request =
                request_builder
                    .build_get_revoc_reg_request(None,
                                                 &_rev_reg_id(),
                                                 TO).unwrap();

            let expected_operation = json!({
                "type": constants::GET_REVOC_REG,
                "revocRegDefId": _rev_reg_id(),
                "timestamp": TO
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }

    mod get_revoc_reg_delta {
        use super::*;

        #[rstest]
        fn test_build_get_revoc_reg_delta_request(request_builder: RequestBuilder) {
            let request =
                request_builder
                    .build_get_revoc_reg_delta_request(None,
                                                       &_rev_reg_id(),
                                                       None,
                                                       TO).unwrap();

            let expected_operation = json!({
                "type": constants::GET_REVOC_REG_DELTA,
                "revocRegDefId": _rev_reg_id(),
                "to": TO
            });

            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_get_revoc_reg_delta_request_for_both_timestamps(request_builder: RequestBuilder) {
            let request =
                request_builder
                    .build_get_revoc_reg_delta_request(None,
                                                       &_rev_reg_id(),
                                                       Some(FROM),
                                                       TO).unwrap();

            let expected_operation = json!({
                "type": constants::GET_REVOC_REG_DELTA,
                "revocRegDefId": _rev_reg_id(),
                "from": FROM,
                "to": TO
            });

            helpers::check_request_operation(&request, expected_operation);
        }
    }
}