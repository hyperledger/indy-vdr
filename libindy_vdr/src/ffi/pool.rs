use crate::common::error::prelude::*;
use crate::pool::{
    PoolBuilder, PoolRunner, PoolTransactions, RequestResult, RequestTarget, TimingResult,
};

use std::collections::{BTreeMap, HashMap};
use std::os::raw::c_char;
use std::sync::RwLock;
use std::thread;

use ffi_support::{rust_string_to_c, FfiStr};

use super::error::{set_last_error, ErrorCode};
use super::requests::{RequestHandle, REQUESTS};
use super::POOL_CONFIG;

new_handle_type!(PoolHandle, FFI_PH_COUNTER);

lazy_static! {
    pub static ref POOLS: RwLock<BTreeMap<PoolHandle, PoolRunner>> = RwLock::new(BTreeMap::new());
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PoolCreateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transactions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transactions_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_weights: Option<HashMap<String, f32>>,
}

#[no_mangle]
pub extern "C" fn indy_vdr_pool_create(params: FfiStr, handle_p: *mut usize) -> ErrorCode {
    catch_err! {
        trace!("Create pool");
        check_useful_c_ptr!(handle_p);
        let params = serde_json::from_str::<PoolCreateParams>(params.as_str())
            .with_input_err("Error deserializing pool create parameters")?;
        let txns = if let Some(txns) = params.transactions {
            PoolTransactions::from_json(txns.as_str())?
        }
        else if let Some(path) = params.transactions_path {
            PoolTransactions::from_json_file(path.as_str())?
        }
        else {
            return Err(input_err(
                "Invalid pool create parameters: must provide transactions or transactions_path"
            ));
        };
        let builder = {
            let gcfg = read_lock!(POOL_CONFIG)?;
            PoolBuilder::from(*gcfg).transactions(txns)?.node_weights(params.node_weights)
        };
        let pool = builder.into_runner()?;
        let handle = PoolHandle::next();
        let mut pools = write_lock!(POOLS)?;
        pools.insert(handle, pool);
        unsafe {
            *handle_p = *handle;
        }
        Ok(ErrorCode::Success)
    }
}

fn handle_pool_refresh(
    pool_handle: PoolHandle,
    old_txns: Vec<String>,
    new_txns: Vec<String>,
) -> ErrorCode {
    catch_err! {
        debug!("Adding {} new pool transactions", new_txns.len());
        let mut txns = PoolTransactions::from_json_transactions(old_txns)?;
        txns.extend_from_json(&new_txns)?;
        let builder = {
            let gcfg = read_lock!(POOL_CONFIG)?;
            PoolBuilder::from(*gcfg)
        };
        let pool = builder.transactions(txns)?.into_runner()?;
        let mut pools = write_lock!(POOLS)?;
        if !pools.contains_key(&pool_handle) {
            let error: VdrError = (VdrErrorKind::Unexpected, "Pool was freed before refresh completed").into();
            let code = ErrorCode::from(error.kind());
            set_last_error(Some(error));
            Ok(code)
        } else {
            pools.insert(pool_handle, pool);
            // previous pool instance will now be dropped
            Ok(ErrorCode::Success)
        }
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_pool_refresh(
    pool_handle: usize,
    cb: Option<extern "C" fn(err: ErrorCode)>,
) -> ErrorCode {
    catch_err! {
        trace!("Refresh pool");
        let cb = cb.ok_or_else(|| input_err("No callback provided"))?;
        let pools = read_lock!(POOLS)?;
        let pool = pools.get(&PoolHandle(pool_handle))
            .ok_or_else(|| input_err("Unknown pool handle"))?;
        pool.refresh(Box::new(
            move |result| {
                let errcode = match result {
                    Ok((old_txns, new_txns, _timing)) => {
                        if let Some(new_txns) = new_txns {
                            // We must spawn a new thread here because this callback
                            // is being run in the PoolRunner's thread, and if we drop
                            // the instance now it will create a deadlock
                            thread::spawn(move || {
                                let result = handle_pool_refresh(PoolHandle(pool_handle), old_txns, new_txns);
                                cb(result)
                            });
                            return
                        } else {
                            ErrorCode::Success
                        }
                    },
                    Err(err) => {
                        let code = ErrorCode::from(err.kind());
                        set_last_error(Some(err));
                        code
                    }
                };
                cb(errcode)
            }))?;
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_pool_get_status(
    pool_handle: usize,
    cb: Option<extern "C" fn(err: ErrorCode, response: *const c_char)>,
) -> ErrorCode {
    catch_err! {
        trace!("Get pool status: {}", pool_handle);
        let cb = cb.ok_or_else(|| input_err("No callback provided"))?;
        let pools = read_lock!(POOLS)?;
        let pool = pools.get(&PoolHandle(pool_handle))
            .ok_or_else(|| input_err("Unknown pool handle"))?;
        pool.get_status(Box::new(
            move |result| {
                let (errcode, reply) = match result {
                    Ok(status) => {
                        let status = status.serialize().unwrap();
                        (ErrorCode::Success, status)
                    },
                    Err(err) => {
                        let code = ErrorCode::from(err.kind());
                        set_last_error(Some(err));
                        (code, String::new())
                    }
                };
                cb(errcode, rust_string_to_c(reply))
            }))?;
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_pool_get_transactions(
    pool_handle: usize,
    cb: Option<extern "C" fn(err: ErrorCode, response: *const c_char)>,
) -> ErrorCode {
    catch_err! {
        trace!("Get pool transactions");
        let cb = cb.ok_or_else(|| input_err("No callback provided"))?;
        let pools = read_lock!(POOLS)?;
        let pool = pools.get(&PoolHandle(pool_handle))
            .ok_or_else(|| input_err("Unknown pool handle"))?;
        pool.get_transactions(Box::new(
            move |result| {
                let (errcode, reply) = match result {
                    Ok(txns) => {
                        (ErrorCode::Success, txns.join("\n"))
                    },
                    Err(err) => {
                        let code = ErrorCode::from(err.kind());
                        set_last_error(Some(err));
                        (code, String::new())
                    }
                };
                cb(errcode, rust_string_to_c(reply))
            }))?;
        Ok(ErrorCode::Success)
    }
}

fn handle_request_result(
    result: VdrResult<(RequestResult<String>, Option<TimingResult>)>,
) -> (ErrorCode, String) {
    match result {
        Ok((reply, _timing)) => match reply {
            RequestResult::Reply(body) => (ErrorCode::Success, body),
            RequestResult::Failed(err) => {
                let code = ErrorCode::from(err.kind());
                set_last_error(Some(err));
                (code, String::new())
            }
        },
        Err(err) => {
            let code = ErrorCode::from(err.kind());
            set_last_error(Some(err));
            (code, String::new())
        }
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_pool_submit_action(
    pool_handle: usize,
    request_handle: usize,
    nodes: FfiStr, // optional
    timeout: i32,  // -1 for default
    cb: Option<extern "C" fn(err: ErrorCode, response: *const c_char)>,
) -> ErrorCode {
    catch_err! {
        trace!("Submit action: {} {} {:?} {}", pool_handle, request_handle, nodes, timeout);
        let cb = cb.ok_or_else(|| input_err("No callback provided"))?;
        let pools = read_lock!(POOLS)?;
        let pool = pools.get(&PoolHandle(pool_handle))
            .ok_or_else(|| input_err("Unknown pool handle"))?;
        let req = {
            let mut reqs = write_lock!(REQUESTS)?;
            reqs.remove(&RequestHandle(request_handle))
                .ok_or_else(|| input_err("Unknown request handle"))?
        };
        let nodes = nodes.as_opt_str().map(serde_json::from_str::<Vec<String>>)
            .transpose().with_input_err("Invalid JSON value for 'nodes'")?;
        let timeout = if timeout == -1 { None } else { Some(timeout as i64) };
        pool.send_request(req, Some(RequestTarget::Full(nodes, timeout)), Box::new(
            move |result| {
                let (errcode, reply) = handle_request_result(result);
                cb(errcode, rust_string_to_c(reply))
            }))?;
        Ok(ErrorCode::Success)
    }
}

#[no_mangle]
pub extern "C" fn indy_vdr_pool_submit_request(
    pool_handle: usize,
    request_handle: usize,
    cb: Option<extern "C" fn(err: ErrorCode, response: *const c_char)>,
) -> ErrorCode {
    catch_err! {
        trace!("Submit request: {} {}", pool_handle, request_handle);
        let cb = cb.ok_or_else(|| input_err("No callback provided"))?;
        let pools = read_lock!(POOLS)?;
        let pool = pools.get(&PoolHandle(pool_handle))
            .ok_or_else(|| input_err("Unknown pool handle"))?;
        let req = {
            let mut reqs = write_lock!(REQUESTS)?;
            reqs.remove(&RequestHandle(request_handle))
                .ok_or_else(|| input_err("Unknown request handle"))?
        };
        pool.send_request(req, None, Box::new(
            move |result| {
                let (errcode, reply) = handle_request_result(result);
                cb(errcode, rust_string_to_c(reply))
            }))?;
        Ok(ErrorCode::Success)
    }
}

// NOTE: at the moment, pending requests are allowed to complete
// and request callbacks are still run, even if we no longer have a
// reference to the pool here. Maybe an optional callback for when
// the close has completed?
#[no_mangle]
pub extern "C" fn indy_vdr_pool_close(pool_handle: usize) -> ErrorCode {
    catch_err! {
        let mut pools = write_lock!(POOLS)?;
        pools.remove(&PoolHandle(pool_handle))
            .ok_or_else(|| input_err("Unknown pool handle"))?;
        Ok(ErrorCode::Success)
    }
}
