extern crate percent_encoding;

use std::cell::RefCell;
use std::rc::Rc;
// use std::time::UNIX_EPOCH;

use hyper::{Body, Method, Request, Response, StatusCode};
use log::trace;
use percent_encoding::percent_decode_str;

use super::AppState;
use indy_vdr::common::did::DidValue;
use indy_vdr::common::error::prelude::*;
use indy_vdr::ledger::requests::cred_def::CredentialDefinitionId;
use indy_vdr::ledger::requests::rev_reg_def::RevocationRegistryId;
use indy_vdr::ledger::requests::schema::SchemaId;
use indy_vdr::pool::helpers::{perform_get_txn, perform_ledger_request};
use indy_vdr::pool::{Pool, RequestResult, TimingResult};

fn format_request_result<T: std::fmt::Display>(
    (result, timing): (RequestResult<T>, Option<TimingResult>),
    pretty: bool,
) -> VdrResult<(String, TimingResult)> {
    match result {
        RequestResult::Reply(message) => {
            let message = message.to_string();
            trace!("Got request response {} {:?}", &message, timing);
            let message = if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&message)
            {
                let result = json["result"].as_object_mut();
                if let Some(result) = result {
                    if pretty {
                        result.remove("identifier");
                        result.remove("reqId");
                        result.remove("state_proof");
                        serde_json::to_string_pretty(&result).unwrap_or(message)
                    } else {
                        serde_json::to_string(&result).unwrap_or(message)
                    }
                } else {
                    message
                }
            } else {
                message
            };
            Ok((message, timing.unwrap()))
        }
        RequestResult::Failed(err) => {
            trace!("No consensus {:?}", timing);
            Err(err)
        }
    }
}

fn format_result<T: std::fmt::Debug>(result: VdrResult<(String, T)>) -> VdrResult<String> {
    Ok(match result {
        Ok((msg, timing)) => format!("{}\n\n{:?}", msg, timing),
        Err(err) => err.to_string(),
    })
}

fn format_ledger_error(err: VdrError) -> Result<Response<Body>, hyper::Error> {
    let msg = err.to_string();
    let (errcode, msg) = match err.into() {
        VdrErrorKind::PoolRequestFailed(failed) => (StatusCode::BAD_REQUEST, failed),
        VdrErrorKind::Input => (StatusCode::BAD_REQUEST, msg),
        VdrErrorKind::PoolTimeout => (StatusCode::GATEWAY_TIMEOUT, msg),
        VdrErrorKind::PoolNoConsensus => (StatusCode::CONFLICT, msg),
        // FIXME - UNAUTHORIZED error when BadRequest msg points to a missing signature
        _ => (StatusCode::INTERNAL_SERVER_ERROR, msg),
    };
    http_status_msg(errcode, msg)
}

/*fn timestamp_now() -> i64 {
    UNIX_EPOCH.elapsed().unwrap().as_secs() as i64
}*/

trait HandleVdrError {
    fn make_response(self) -> Result<Response<Body>, hyper::Error>;
}

impl<T> HandleVdrError for VdrResult<T>
where
    Body: From<T>,
{
    fn make_response(self) -> Result<Response<Body>, hyper::Error> {
        match self {
            Err(err) => format_ledger_error(err),
            Ok(msg) => Ok(Response::builder().body(Body::from(msg)).unwrap()),
        }
    }
}

fn http_status(code: StatusCode) -> Result<Response<Body>, hyper::Error> {
    http_status_msg(code, "")
}

fn http_status_msg<M>(code: StatusCode, msg: M) -> Result<Response<Body>, hyper::Error>
where
    Body: From<M>,
{
    Ok(Response::builder()
        .status(code)
        .body(Body::from(msg))
        .unwrap())
}

async fn get_pool_genesis<T: Pool>(pool: &T) -> VdrResult<String> {
    let txns = pool.get_transactions()?;
    Ok(txns.join("\n"))
}

fn format_pool_status(state: Rc<RefCell<AppState>>) -> VdrResult<String> {
    let opt_pool = &state.borrow().pool;
    let (status, mt_root, mt_size, nodes) = if let Some(pool) = opt_pool {
        let (mt_root, mt_size) = pool.get_merkle_tree_root();
        let nodes = pool.get_node_aliases();
        ("active", Some(mt_root), Some(mt_size), Some(nodes))
    } else {
        ("init", None, None, None)
    };
    let last_refresh = &state.borrow().last_refresh;
    let last_refresh = last_refresh.map(|tm| tm.elapsed().map(|d| d.as_secs()).ok());

    let result = json!({"status": status, "pool_mt_root": mt_root, "pool_mt_size": mt_size, "pool_nodes": nodes, "last_refresh": last_refresh});
    Ok(serde_json::to_string(&result)
        .with_err_msg(VdrErrorKind::Unexpected, "Error serializing JSON")?)
}

async fn get_attrib<T: Pool>(pool: &T, dest: &str, raw: &str, pretty: bool) -> VdrResult<String> {
    let dest = DidValue::from_str(dest)?;
    let request = pool.get_request_builder().build_get_attrib_request(
        None,
        &dest,
        Some(raw.to_string()),
        None,
        None,
    )?;
    let result = perform_ledger_request(pool, request, None).await?;
    format_result(format_request_result(result, pretty))
}

async fn get_nym<T: Pool>(pool: &T, nym: &str, pretty: bool) -> VdrResult<String> {
    let nym = DidValue::from_str(nym)?;
    let request = pool
        .get_request_builder()
        .build_get_nym_request(None, &nym)?;
    let result = perform_ledger_request(pool, request, None).await?;
    format_result(format_request_result(result, pretty))
}

async fn get_schema<T: Pool>(pool: &T, schema_id: &str, pretty: bool) -> VdrResult<String> {
    let schema_id = SchemaId::from_str(schema_id)?;
    let request = pool
        .get_request_builder()
        .build_get_schema_request(None, &schema_id)?;
    let result = perform_ledger_request(pool, request, None).await?;
    format_result(format_request_result(result, pretty))
}

async fn get_cred_def<T: Pool>(pool: &T, cred_def_id: &str, pretty: bool) -> VdrResult<String> {
    let cred_def_id = CredentialDefinitionId::from_str(cred_def_id)?;
    let request = pool
        .get_request_builder()
        .build_get_cred_def_request(None, &cred_def_id)?;
    let result = perform_ledger_request(pool, request, None).await?;
    format_result(format_request_result(result, pretty))
}

async fn get_revoc_reg_def<T: Pool>(
    pool: &T,
    revoc_reg_def_id: &str,
    pretty: bool,
) -> VdrResult<String> {
    let revoc_reg_def_id = RevocationRegistryId::from_str(revoc_reg_def_id)?;
    let request = pool
        .get_request_builder()
        .build_get_revoc_reg_def_request(None, &revoc_reg_def_id)?;
    let result = perform_ledger_request(pool, request, None).await?;
    format_result(format_request_result(result, pretty))
}

/*
async fn get_revoc_reg<T: Pool>(
    pool: &T,
    revoc_reg_def_id: &str,
    pretty: bool,
) -> VdrResult<String> {
    let revoc_reg_def_id = RevocationRegistryId::from_str(revoc_reg_def_id)?;
    let request = pool.get_request_builder().build_get_revoc_reg_request(
        None,
        &revoc_reg_def_id,
        timestamp_now(),
    )?;
    let result = perform_ledger_request(pool, request, None).await?;
    format_result(format_request_result(result, pretty))
}

async fn get_revoc_reg_delta<T: Pool>(
    pool: &T,
    revoc_reg_def_id: &str,
    pretty: bool,
) -> VdrResult<String> {
    let revoc_reg_def_id = RevocationRegistryId::from_str(revoc_reg_def_id)?;
    let request = pool
        .get_request_builder()
        .build_get_revoc_reg_delta_request(None, &revoc_reg_def_id, Some(0), timestamp_now())?;
    let result = perform_ledger_request(pool, request, None).await?;
    format_result(format_request_result(result, pretty))
}
*/

/*
async fn test_get_validator_info<T: Pool>(pool: &T, pretty: bool) -> VdrResult<String> {
    let result = perform_get_validator_info(pool).await?;
    format_result(format_request_result(result, pretty))
}
*/

async fn get_taa<T: Pool>(pool: &T, pretty: bool) -> VdrResult<String> {
    let request = pool
        .get_request_builder()
        .build_get_txn_author_agreement_request(None, None)?;
    let result = perform_ledger_request(pool, request, None).await?;
    format_result(format_request_result(result, pretty))
}

async fn get_aml<T: Pool>(pool: &T, pretty: bool) -> VdrResult<String> {
    let request = pool
        .get_request_builder()
        .build_get_acceptance_mechanisms_request(None, None, None)?;
    let result = perform_ledger_request(pool, request, None).await?;
    format_result(format_request_result(result, pretty))
}

async fn get_auth_rule<T: Pool>(
    pool: &T,
    auth_type: Option<String>,
    auth_action: Option<String>,
    field: Option<String>,
    pretty: bool,
) -> VdrResult<String> {
    let request = pool.get_request_builder().build_get_auth_rule_request(
        None,
        auth_type,
        auth_action,
        field,
        None,
        None,
    )?;
    let result = perform_ledger_request(pool, request, None).await?;
    format_result(format_request_result(result, pretty))
}

async fn get_txn<T: Pool>(pool: &T, ledger: i32, seq_no: i32, pretty: bool) -> VdrResult<String> {
    let result = perform_get_txn(pool, ledger, seq_no).await?;
    format_result(format_request_result(result, pretty))
}

async fn submit_request<T: Pool>(
    pool: &T,
    message: Vec<u8>,
    pretty: bool,
) -> VdrResult<(String, String)> {
    let (request, target) =
        pool.get_request_builder()
            .build_custom_request(&message, None, (None, None))?;
    let result = perform_ledger_request(pool, request, target).await?;
    let (response, timing) = format_request_result(result, pretty)?;
    Ok((response, format!("{:?}", timing)))
}

pub async fn handle_request<T: Pool>(
    req: Request<Body>,
    state: Rc<RefCell<AppState>>,
) -> Result<Response<Body>, hyper::Error> {
    let mut parts = req
        .uri()
        .path()
        .split('/')
        .map(percent_decode_str)
        .flat_map(|part| {
            part.decode_utf8()
                .map(|p| p.into_owned())
                .ok()
                .filter(|p| !p.is_empty())
        });
    let pretty = req.uri().query() == Some("fmt");
    let fst = parts.next().unwrap_or_else(|| "".to_owned());
    let req_method = req.method();
    if (req_method, fst.is_empty()) == (&Method::GET, true) {
        return format_pool_status(state.clone()).make_response();
    }
    let opt_pool = &state.borrow().pool;
    let pool = match opt_pool {
        None => {
            return http_status(StatusCode::SERVICE_UNAVAILABLE);
        }
        Some(pool) => pool,
    };
    match (req_method, fst.as_str()) {
        // (&Method::GET, "status") => test_get_validator_info(pool, pretty).await.make_response(),
        (&Method::GET, "submit") => http_status(StatusCode::METHOD_NOT_ALLOWED),
        (&Method::POST, "submit") => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
            let body = body_bytes.iter().cloned().collect::<Vec<u8>>();
            if !body.is_empty() {
                match submit_request(pool, body, pretty).await {
                    Ok((result, timing)) => {
                        let mut response = Response::new(Body::from(result));
                        response
                            .headers_mut()
                            .append("X-Requests", timing.parse().unwrap());
                        Ok(response)
                    }
                    Err(err) => format_ledger_error(err),
                }
            } else {
                http_status(StatusCode::BAD_REQUEST)
            }
        }
        (&Method::GET, "genesis") => get_pool_genesis(pool).await.make_response(),
        (&Method::GET, "taa") => get_taa(pool, pretty).await.make_response(),
        (&Method::GET, "aml") => get_aml(pool, pretty).await.make_response(),
        (&Method::GET, "attrib") => {
            if let (Some(dest), Some(attrib)) = (parts.next(), parts.next()) {
                // NOTE: 'endpoint' is currently the only supported attribute
                get_attrib(pool, &*dest, &*attrib, pretty)
                    .await
                    .make_response()
            } else {
                http_status(StatusCode::NOT_FOUND)
            }
        }
        (&Method::GET, "auth") => {
            if let Some(auth_type) = parts.next() {
                if let Some(auth_action) = parts.next() {
                    get_auth_rule(
                        pool,
                        Some(auth_type.to_owned()),
                        Some(auth_action.to_owned()),
                        Some("*".to_owned()),
                        pretty,
                    )
                    .await
                    .make_response()
                } else {
                    http_status(StatusCode::NOT_FOUND)
                }
            } else {
                get_auth_rule(pool, None, None, None, pretty)
                    .await
                    .make_response() // get all
            }
        }
        (&Method::GET, "cred_def") => {
            if let Some(cred_def_id) = parts.next() {
                get_cred_def(pool, &*cred_def_id, pretty)
                    .await
                    .make_response()
            } else {
                http_status(StatusCode::NOT_FOUND)
            }
        }
        (&Method::GET, "nym") => {
            if let Some(nym) = parts.next() {
                get_nym(pool, &*nym, pretty).await.make_response()
            } else {
                http_status(StatusCode::NOT_FOUND)
            }
        }
        (&Method::GET, "rev_reg_def") => {
            if let Some(rev_reg_def_id) = parts.next() {
                get_revoc_reg_def(pool, &*rev_reg_def_id, pretty)
                    .await
                    .make_response()
            } else {
                http_status(StatusCode::NOT_FOUND)
            }
        }
        /*(&Method::GET, "rev_reg") => {
            if let Some(rev_reg_def_id) = parts.next() {
                get_revoc_reg(pool, &*rev_reg_def_id, pretty)
                    .await
                    .make_response()
            } else {
                http_status(StatusCode::NOT_FOUND)
            }
        }
        (&Method::GET, "rev_reg_delta") => {
            if let Some(rev_reg_def_id) = parts.next() {
                get_revoc_reg_delta(pool, &*rev_reg_def_id, pretty)
                    .await
                    .make_response()
            } else {
                http_status(StatusCode::NOT_FOUND)
            }
        }*/
        (&Method::GET, "schema") => {
            if let Some(schema_id) = parts.next() {
                get_schema(pool, &*schema_id, pretty).await.make_response()
            } else {
                http_status(StatusCode::NOT_FOUND)
            }
        }
        (&Method::GET, "txn") => {
            if let (Some(ledger), Some(txn)) = (parts.next(), parts.next()) {
                if let (Ok(ledger), Ok(txn)) = (ledger.parse::<i32>(), txn.parse::<i32>()) {
                    return get_txn(pool, ledger, txn, pretty).await.make_response();
                }
            }
            http_status(StatusCode::NOT_FOUND)
        }
        (&Method::GET, _) => http_status(StatusCode::NOT_FOUND),
        _ => http_status(StatusCode::METHOD_NOT_ALLOWED),
    }
}
