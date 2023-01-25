use crate::common::error::prelude::*;
#[cfg(any(feature = "rich_schema", test))]
use crate::ledger::identifiers::RichSchemaId;
use crate::ledger::identifiers::{CredentialDefinitionId, RevocationRegistryId, SchemaId};
use crate::ledger::requests::auth_rule::{AuthRules, Constraint};
use crate::ledger::requests::author_agreement::{AcceptanceMechanisms, GetTxnAuthorAgreementData};
use crate::ledger::requests::cred_def::CredentialDefinition;
use crate::ledger::requests::node::NodeOperationData;
use crate::ledger::requests::rev_reg::RevocationRegistryDelta;
use crate::ledger::requests::rev_reg_def::{RegistryType, RevocationRegistryDefinition};
#[cfg(any(feature = "rich_schema", test))]
use crate::ledger::requests::rich_schema::RSContent;
use crate::ledger::requests::schema::Schema;
use crate::pool::PreparedRequest;
use crate::utils::did::DidValue;
use crate::utils::Qualifiable;

use ffi_support::FfiStr;

use super::error::{set_last_error, ErrorCode};
use super::requests::{add_request, get_request_builder, RequestHandle};

#[no_mangle]
pub extern "C" fn indy_vdr_build_acceptance_mechanisms_request(
    submitter_did: FfiStr,
    aml: FfiStr,
    version: FfiStr,
    aml_context: FfiStr, // optional
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build TXN_AUTHR_AGRMT_AML request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(submitter_did.as_str())?;
        let aml = serde_json::from_str::<AcceptanceMechanisms>(aml.as_str())
            .with_input_err("Error deserializing AcceptanceMechanisms")?;
        let version = version.into_string();
        let aml_context = aml_context.into_opt_string();
        let req = builder.build_acceptance_mechanisms_request(&identifier, aml, version, aml_context)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_acceptance_mechanisms_request(
    submitter_did: FfiStr, // optional
    timestamp: i64,
    version: FfiStr, // optional
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_TXN_AUTHR_AGRMT_AML request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = submitter_did
            .as_opt_str()
            .map(DidValue::from_str)
            .transpose()?;
        let timestamp = if timestamp == -1 { None } else { Some(timestamp as u64) };
        let version = version.into_opt_string();
        let req = builder.build_get_acceptance_mechanisms_request(identifier.as_ref(), timestamp, version)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_attrib_request(
    submitter_did: FfiStr, // optional
    target_did: FfiStr,
    hash: FfiStr, // optional
    raw: FfiStr,  // optional
    enc: FfiStr,  // optional
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build ATTRIB request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(submitter_did.as_str())?;
        let dest = DidValue::from_str(target_did.as_str())?;
        let raw = match raw.as_opt_str() {
            Some(s) => {
                let js = serde_json::from_str(s).with_input_err("Error deserializing raw value as JSON")?;
                Some(js)
            }
            None => None,
        };
        let hash = hash.into_opt_string();
        let enc = enc.into_opt_string();
        let req = builder.build_attrib_request(&identifier, &dest, hash, raw.as_ref(), enc)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_attrib_request(
    submitter_did: FfiStr, // optional
    target_did: FfiStr,
    raw: FfiStr,  // optional
    hash: FfiStr, // optional
    enc: FfiStr,  // optional
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_ATTRIB request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = submitter_did
            .as_opt_str()
            .map(DidValue::from_str)
            .transpose()?;
        let dest = DidValue::from_str(target_did.as_str())?;
        let raw = raw.into_opt_string();
        let hash = hash.into_opt_string();
        let enc = enc.into_opt_string();
        let req = builder.build_get_attrib_request(identifier.as_ref(), &dest, raw, hash, enc)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_cred_def_request(
    submitter_did: FfiStr,
    cred_def: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build CRED_DEF request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(submitter_did.as_str())?;
        let cred_def = serde_json::from_str::<CredentialDefinition>(cred_def.as_str())
            .with_input_err("Error deserializing CredentialDefinition")?;
        let req = builder.build_cred_def_request(&identifier, cred_def)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_custom_request(
    request_json: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build custom pool request");
        check_useful_c_ptr!(handle_p);
        let request = PreparedRequest::from_request_json(request_json.as_str())?;
        let handle = add_request(request)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_disable_all_txn_author_agreements_request(
    submitter_did: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build DISABLE_ALL_TXN_AUTHR_AGRMTS request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(submitter_did.as_str())?;
        let req = builder.build_disable_all_txn_author_agreements_request(&identifier)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_cred_def_request(
    submitter_did: FfiStr, // optional
    cred_def_id: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_CRED_DEF request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = submitter_did.as_opt_str().map(DidValue::from_str).transpose()?;
        let cred_def_id = CredentialDefinitionId::from_str(cred_def_id.as_str())?;
        let req = builder.build_get_cred_def_request(identifier.as_ref(), &cred_def_id)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_nym_request(
    submitter_did: FfiStr, // optional
    dest: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_NYM request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = submitter_did.as_opt_str().map(DidValue::from_str).transpose()?;
        let dest = DidValue::from_str(dest.as_str())?;
        let req = builder.build_get_nym_request(identifier.as_ref(), &dest)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_revoc_reg_def_request(
    submitter_did: FfiStr, // optional
    revoc_reg_id: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_REVOC_REG_DEF request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = submitter_did.as_opt_str().map(DidValue::from_str).transpose()?;
        let revoc_reg_id = RevocationRegistryId::from_str(revoc_reg_id.as_str())?;
        let req = builder.build_get_revoc_reg_def_request(identifier.as_ref(), &revoc_reg_id)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_revoc_reg_request(
    submitter_did: FfiStr, // optional
    revoc_reg_id: FfiStr,
    timestamp: i64,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_REVOC_REG request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = submitter_did.as_opt_str().map(DidValue::from_str).transpose()?;
        let revoc_reg_id = RevocationRegistryId::from_str(revoc_reg_id.as_str())?;
        let req = builder.build_get_revoc_reg_request(identifier.as_ref(), &revoc_reg_id, timestamp)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_revoc_reg_delta_request(
    submitter_did: FfiStr, // optional
    revoc_reg_id: FfiStr,
    from_ts: i64, // -1 for none
    to_ts: i64,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_REVOC_REG_DELTA request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = submitter_did.as_opt_str().map(DidValue::from_str).transpose()?;
        let revoc_reg_id = RevocationRegistryId::from_str(revoc_reg_id.as_str())?;
        let from_ts = if from_ts == -1 {None} else {Some(from_ts)};
        let req = builder.build_get_revoc_reg_delta_request(identifier.as_ref(), &revoc_reg_id, from_ts, to_ts)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_schema_request(
    submitter_did: FfiStr, // optional
    schema_id: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_SCHEMA request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = submitter_did.as_opt_str().map(DidValue::from_str).transpose()?;
        let schema_id = SchemaId::from_str(schema_id.as_str())?;
        let req = builder.build_get_schema_request(identifier.as_ref(), &schema_id)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_txn_author_agreement_request(
    submitter_did: FfiStr, // optional
    data: FfiStr,          // optional
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_TXN_AUTHR_AGRMT request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = submitter_did
            .as_opt_str()
            .map(DidValue::from_str)
            .transpose()?;
        let data = match data.as_opt_str() {
            Some(data)  => Some(serde_json::from_str::<GetTxnAuthorAgreementData>(data)
                .with_input_err("Error deserializing GetTxnAuthorAgreementData")?),
            None => None
        };
        let req = builder.build_get_txn_author_agreement_request(identifier.as_ref(), data.as_ref())?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_txn_request(
    submitter_did: FfiStr, // optional
    ledger_type: i32,
    seq_no: i32,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_TXN request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = submitter_did.as_opt_str().map(DidValue::from_str).transpose()?;
        let req = builder.build_get_txn_request(identifier.as_ref(), ledger_type, seq_no)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_validator_info_request(
    submitter_did: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_VALIDATOR_INFO request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(submitter_did.as_str())?;
        let req = builder.build_get_validator_info_request(&identifier)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_nym_request(
    submitter_did: FfiStr,
    dest: FfiStr,
    verkey: FfiStr, // optional
    alias: FfiStr,  // optional
    role: FfiStr,   // optional
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build NYM request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(submitter_did.as_str())?;
        let dest = DidValue::from_str(dest.as_str())?;
        let verkey = verkey.into_opt_string();
        let alias = alias.into_opt_string();
        let role = role.into_opt_string();
        let req = builder.build_nym_request(&identifier, &dest, verkey, alias, role)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_revoc_reg_def_request(
    submitter_did: FfiStr,
    revoc_reg_def: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build REVOC_REG_DEF request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(submitter_did.as_str())?;
        let revoc_reg_def = serde_json::from_str::<RevocationRegistryDefinition>(revoc_reg_def.as_str())
            .with_input_err("Error deserializing RevocationRegistryDefinition")?;
        let req = builder.build_revoc_reg_def_request(&identifier, revoc_reg_def)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_revoc_reg_entry_request(
    submitter_did: FfiStr,
    revoc_reg_def_id: FfiStr,
    revoc_reg_def_type: FfiStr,
    revoc_reg_entry: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build REVOC_REG_ENTRY request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(submitter_did.as_str())?;
        let revoc_reg_def_id = RevocationRegistryId::from_str(revoc_reg_def_id.as_str())?;
        let revoc_reg_def_type = RegistryType::from_str(revoc_reg_def_type.as_str())?;
        let revoc_reg_entry = serde_json::from_str::<RevocationRegistryDelta>(revoc_reg_entry.as_str())
            .with_input_err("Error deserializing RevocationRegistryDelta")?;
        let req = builder.build_revoc_reg_entry_request(&identifier, &revoc_reg_def_id, &revoc_reg_def_type, revoc_reg_entry)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_schema_request(
    submitter_did: FfiStr,
    schema: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build SCHEMA request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(submitter_did.as_str())?;
        let schema = serde_json::from_str::<Schema>(schema.as_str())
            .with_input_err("Error deserializing Schema")?;
        let req = builder.build_schema_request(&identifier, schema)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_txn_author_agreement_request(
    submitter_did: FfiStr,
    text: FfiStr,
    version: FfiStr,
    ratification_ts: i64,
    retirement_ts: i64,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build TXN_AUTHR_AGRMT request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(submitter_did.as_str())?;
        let text = text.into_opt_string();
        let version = version.into_string();
        let ratification_ts = if ratification_ts == -1 { None } else { Some(ratification_ts as u64) };
        let retirement_ts = if retirement_ts == -1 { None } else { Some(retirement_ts as u64) };
        let req = builder.build_txn_author_agreement_request(
            &identifier, text, version, ratification_ts, retirement_ts
        )?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[cfg(any(feature = "rich_schema", test))]
#[no_mangle]
pub extern "C" fn indy_vdr_build_rich_schema_request(
    submitter_did: FfiStr,
    rs_id: FfiStr,
    rs_content: FfiStr,
    rs_name: FfiStr,
    rs_version: FfiStr,
    rs_type: FfiStr,
    ver: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build RICH_SCHEMA request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(submitter_did.as_str())?;
        let rs_id = RichSchemaId::from_str(rs_id.as_str())?;
        let rs_content = rs_content.into_string();
        let rs_name = rs_name.into_string();
        let rs_version = rs_version.into_string();
        let rs_type = rs_type.into_string();
        let ver = ver.into_string();
        let req = builder.build_rich_schema_request(
            &identifier,
            rs_id,
            RSContent(rs_content),
            rs_name,
            rs_version,
            rs_type,
            ver
        )?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[cfg(any(feature = "rich_schema", test))]
#[no_mangle]
pub extern "C" fn indy_vdr_build_get_rich_schema_object_by_id_request(
    submitter_did: FfiStr,
    rs_id: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_RICH_SCHEMA_BY_ID request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(submitter_did.as_str())?;
        let rs_id = RichSchemaId::from_str(rs_id.as_str())?;
        let req = builder.build_get_rich_schema_by_id(&identifier, &rs_id)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[cfg(any(feature = "rich_schema", test))]
#[no_mangle]
pub extern "C" fn indy_vdr_build_get_rich_schema_object_by_metadata_request(
    submitter_did: FfiStr,
    rs_type: FfiStr,
    rs_name: FfiStr,
    rs_version: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_RICH_SCHEMA_BY_METADATA request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(submitter_did.as_str())?;
        let rs_type = rs_type.into_string();
        let rs_name = rs_name.into_string();
        let rs_version = rs_version.into_string();
        let req = builder.build_get_rich_schema_by_metadata(&identifier, rs_type, rs_name, rs_version)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_node_request(
    identifier: FfiStr,
    dest: FfiStr,
    data: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build NODE request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(identifier.as_str())?;
        let dest = DidValue::from_str(dest.as_str())?;
        let node_data = serde_json::from_str::<NodeOperationData>(data.as_str())
            .with_input_err("Error deserializing NodeData")?;
        let req = builder.build_node_request(&identifier, &dest, node_data)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_pool_config_request(
    identifier: FfiStr,
    writes: i8,
    force: i8,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build POOL_CONFIG request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(identifier.as_str())?;
        let req = builder.build_pool_config_request(&identifier, writes != 0, force != 0)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_pool_restart_request(
    identifier: FfiStr,
    action: FfiStr,
    datetime: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build POOL_RESTART request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(identifier.as_str())?;
        let action = action.as_str();
        let datetime = datetime.into_opt_string();
        let req = builder.build_pool_restart_request(&identifier, action, datetime.as_deref())?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_auth_rule_request(
    submitter_did: FfiStr,
    txn_type: FfiStr,
    action: FfiStr,
    field: FfiStr,
    old_value: FfiStr,
    new_value: FfiStr,
    constraint: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build AUTH_RULE request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let submitter_did = DidValue::from_str(submitter_did.as_str())?;
        let txn_type = txn_type.as_str();
        let action = action.as_str();
        let field = field.as_str();
        let old_value = old_value.into_opt_string();
        let new_value = new_value.into_opt_string();
        let constraint = serde_json::from_str::<Constraint>(constraint.as_str())
            .with_input_err("Error deserializing Constraint")?;
        let req = builder.build_auth_rule_request(&submitter_did, txn_type.to_string(), action.to_string(),
                                                   field.to_string(), old_value, new_value, constraint)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_auth_rules_request(
    submitter_did: FfiStr,
    rules: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build AUTH_RULES request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let submitter_did = DidValue::from_str(submitter_did.as_str())?;
        let rules = serde_json::from_str::<AuthRules>(rules.as_str())
            .with_input_err("Error deserializing AuthRules")?;
        let req = builder.build_auth_rules_request(&submitter_did, rules)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_auth_rule_request(
    submitter_did: FfiStr,
    auth_type: FfiStr,
    auth_action: FfiStr,
    field: FfiStr,
    old_value: FfiStr,
    new_value: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_AUTH_RULE request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let submitter_did = submitter_did.as_opt_str().map(DidValue::from_str).transpose()?;
        let auth_type = auth_type.into_opt_string();
        let auth_action = auth_action.into_opt_string();
        let field = field.into_opt_string();
        let old_value = old_value.into_opt_string();
        let new_value = new_value.into_opt_string();
        let req = builder.build_get_auth_rule_request(submitter_did.as_ref(), auth_type, auth_action, field, old_value, new_value)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_ledgers_freeze_request(
    identifier: FfiStr,
    ledgers_ids: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build LEDGERS_FREEZE request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(identifier.as_str())?;
        let ledgers_ids = serde_json::from_str::<Vec<u64>>(ledgers_ids.as_str())
            .with_input_err("Error deserializing LedgerIDs")?;
        let req = builder.build_ledgers_freeze_request(&identifier, &ledgers_ids)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_frozen_ledgers_request(
    identifier: FfiStr,
    handle_p: *mut RequestHandle,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_FROZEN_LEDGERS request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(identifier.as_str())?;
        let req = builder.build_get_frozen_ledgers_request(&identifier)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = handle;
        }
        Ok(ErrorCode::Success)
    }
}
