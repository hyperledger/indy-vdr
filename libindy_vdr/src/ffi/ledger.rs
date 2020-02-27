use crate::common::did::DidValue;
use crate::common::error::prelude::*;
use crate::ledger::identifiers::cred_def::CredentialDefinitionId;
use crate::ledger::identifiers::rev_reg_def::RevocationRegistryId;
use crate::ledger::identifiers::schema::SchemaId;
use crate::ledger::requests::author_agreement::{AcceptanceMechanisms, GetTxnAuthorAgreementData};

use ffi_support::FfiStr;

use super::error::{set_last_error, ErrorCode};
use super::requests::{add_request, get_request_builder};

#[no_mangle]
pub extern "C" fn indy_vdr_build_acceptance_mechanisms_request(
    submitter_did: FfiStr,
    aml: FfiStr,
    version: FfiStr,
    aml_context: FfiStr, // optional
    handle_p: *mut usize,
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
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_acceptance_mechanisms_request(
    submitter_did: FfiStr, // optional
    timestamp: i64,
    version: FfiStr, // optional
    handle_p: *mut usize,
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
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_custom_request(
    request_json: FfiStr,
    handle_p: *mut usize,
) -> ErrorCode {
    catch_err! {
        trace!("Build custom pool request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let (req, _target) = builder.build_custom_request_from_str(
            request_json.as_str(), None, (None, None)
        )?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_disable_all_txn_author_agreements_request(
    submitter_did: FfiStr,
    handle_p: *mut usize,
) -> ErrorCode {
    catch_err! {
        trace!("Build DISABLE_ALL_TXN_AUTHR_AGRMTS request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(submitter_did.as_str())?;
        let req = builder.build_disable_all_txn_author_agreements_request(&identifier)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_cred_def_request(
    submitter_did: FfiStr, // optional
    cred_def_id: FfiStr,
    handle_p: *mut usize,
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
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_nym_request(
    submitter_did: FfiStr, // optional
    dest: FfiStr,
    handle_p: *mut usize,
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
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_revoc_reg_def_request(
    submitter_did: FfiStr, // optional
    revoc_reg_id: FfiStr,
    handle_p: *mut usize,
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
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_revoc_reg_request(
    submitter_did: FfiStr, // optional
    revoc_reg_id: FfiStr,
    timestamp: i64,
    handle_p: *mut usize,
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
            *handle_p = *handle;
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
    handle_p: *mut usize,
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
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_schema_request(
    submitter_did: FfiStr, // optional
    schema_id: FfiStr,
    handle_p: *mut usize,
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
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_txn_author_agreement_request(
    submitter_did: FfiStr, // optional
    data: FfiStr,          // optional
    handle_p: *mut usize,
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
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_txn_request(
    submitter_did: FfiStr, // optional
    ledger_type: i32,
    seq_no: i32,
    handle_p: *mut usize,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_TXN request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = submitter_did.as_opt_str().map(DidValue::from_str).transpose()?;
        let req = builder.build_get_txn_request(identifier.as_ref(), ledger_type, seq_no)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_build_get_validator_info_request(
    submitter_did: FfiStr,
    handle_p: *mut usize,
) -> ErrorCode {
    catch_err! {
        trace!("Build GET_VALIDATOR_INFO request");
        check_useful_c_ptr!(handle_p);
        let builder = get_request_builder()?;
        let identifier = DidValue::from_str(submitter_did.as_str())?;
        let req = builder.build_get_validator_info_request(&identifier)?;
        let handle = add_request(req)?;
        unsafe {
            *handle_p = *handle;
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
    handle_p: *mut usize,
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
            *handle_p = *handle;
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
    handle_p: *mut usize,
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
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}
