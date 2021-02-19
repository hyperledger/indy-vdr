extern crate percent_encoding;

use std::cell::RefCell;
use std::convert::TryFrom;
use std::rc::Rc;
use std::time::UNIX_EPOCH;

use hyper::{Body, Method, Request, Response, StatusCode};
use percent_encoding::percent_decode_str;

use super::AppState;
use indy_vdr::common::error::prelude::*;
use indy_vdr::ledger::identifiers::{CredentialDefinitionId, RevocationRegistryId, SchemaId};
use indy_vdr::pool::helpers::{perform_get_txn, perform_ledger_request};
use indy_vdr::pool::{LedgerType, Pool, PreparedRequest, RequestResult, TimingResult};
use indy_vdr::utils::did::DidValue;
use indy_vdr::utils::Qualifiable;

#[derive(PartialEq, Eq)]
enum ResponseFormat {
    Html,
    Raw,
}

enum ResponseType {
    Genesis(String),
    Json(String),
    RequestReply(String, Option<TimingResult>),
    RequestFailed(VdrError, Option<TimingResult>),
    Status(StatusCode, String),
}

impl<T> From<(RequestResult<T>, Option<TimingResult>)> for ResponseType
where
    T: std::fmt::Display,
{
    fn from(result: (RequestResult<T>, Option<TimingResult>)) -> ResponseType {
        match result {
            (RequestResult::Reply(message), timing) => {
                ResponseType::RequestReply(message.to_string(), timing)
            }
            (RequestResult::Failed(err), timing) => ResponseType::RequestFailed(err, timing),
        }
    }
}

impl From<VdrError> for ResponseType {
    fn from(err: VdrError) -> ResponseType {
        let (errcode, msg) = convert_error(err);
        ResponseType::Status(errcode, msg)
    }
}

fn convert_error(err: VdrError) -> (StatusCode, String) {
    let msg = err.to_string();
    match err.into() {
        VdrErrorKind::PoolRequestFailed(failed) => (StatusCode::BAD_REQUEST, failed),
        VdrErrorKind::Input => (StatusCode::BAD_REQUEST, msg),
        VdrErrorKind::PoolTimeout => (StatusCode::GATEWAY_TIMEOUT, msg),
        VdrErrorKind::PoolNoConsensus => (StatusCode::CONFLICT, msg),
        // FIXME - UNAUTHORIZED error when BadRequest msg points to a missing signature
        _ => (StatusCode::INTERNAL_SERVER_ERROR, msg),
    }
}

fn format_json_reply(message: String, pretty: bool) -> String {
    if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&message) {
        let result = json["result"].as_object_mut();
        if let Some(result) = result {
            if pretty {
                result.remove("identifier");
                result.remove("reqId");
                result.remove("state_proof");
                serde_json::to_string_pretty(&json).unwrap_or(message)
            } else {
                serde_json::to_string(&json).unwrap_or(message)
            }
        } else {
            message
        }
    } else {
        message
    }
}

pub fn escape_html(val: &str) -> String {
    val.replace("&", "&amp;").replace("<", "&lt;")
}

fn html_template(main: String, timing: Option<TimingResult>) -> String {
    let main = main
        .lines()
        .map(|line| {
            let mut ws = line.len();
            let line = line.trim_start();
            ws -= line.len();
            format!(
                "<div style=\"padding-left:{}ch;text-indent:-{}ch\">{}{}</div>\n",
                ws + 2,
                ws + 2,
                "&nbsp;".repeat(ws),
                escape_html(line)
            )
        })
        .collect::<Vec<String>>()
        .join("");

    let timing = if let Some(timing) = timing {
        format!(
            "<div class=\"timing\"><div class=\"code\">{:?}</div></div>",
            timing
        )
    } else {
        "".to_owned()
    };

    format!(
        r#"
<html>
    <head>
        <meta charset="utf-8">
        <title>Indy-VDR</title>
        <style type="text/css">
            .response::before, .timing::before {{
                background: #eee;
                border-radius: 4px 4px 0 0;
                content: "Response";
                display: block;
                font-family: system-ui, sans-serif;
                padding: 2px 0.5em;
            }}
            .timing::before {{
                content: "Timing";
            }}
            .response, .timing {{
                border: 1px solid #ddd;
                border-radius: 5px;
                margin: 1.5em 5%;
            }}
            .code {{
                font-family: monospace;
                font-size: 115%;
                padding: 0.5em;
                word-break: break-all;
            }}
        </style>
    </head>
    <body>
        <div class="response">
            <div class="code">{}</div>
        </div>
        {}
    </body>
</html>
    "#,
        main, timing
    )
}

fn format_text(
    result: String,
    format: ResponseFormat,
    status: StatusCode,
    timing: Option<TimingResult>,
) -> Response<Body> {
    if format == ResponseFormat::Html {
        Response::builder()
            .status(status)
            .header("Content-Type", "text/html")
            .body(html_template(result, timing).into())
            .unwrap()
    } else {
        Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .header(
                "X-Requests",
                if let Some(timing) = timing {
                    format!("{:?}", timing)
                } else {
                    "".to_owned()
                },
            )
            .body(result.into())
            .unwrap()
    }
}

fn format_result(
    result: VdrResult<ResponseType>,
    format: ResponseFormat,
) -> Result<Response<Body>, hyper::Error> {
    let result = match result {
        Ok(result) => result,
        Err(err) => err.into(),
    };
    let pretty = format == ResponseFormat::Html;
    let response = match result {
        ResponseType::Genesis(genesis) => format_text(genesis, format, StatusCode::OK, None),
        ResponseType::Json(json) => {
            let body = if pretty {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json.as_str()) {
                    serde_json::to_string_pretty(&parsed).unwrap_or(json)
                } else {
                    json
                }
            } else {
                json
            };
            format_text(body, format, StatusCode::OK, None)
        }
        ResponseType::RequestReply(reply, timing) => {
            let reply = format_json_reply(reply, pretty);
            format_text(reply, format, StatusCode::OK, timing)
        }
        ResponseType::RequestFailed(err, timing) => {
            let (errcode, msg) = convert_error(err);
            format_text(msg, format, errcode, timing)
        }
        ResponseType::Status(code, msg) => format_text(msg, format, code, None),
    };
    Ok(response)
}

fn timestamp_now() -> i64 {
    UNIX_EPOCH.elapsed().unwrap().as_secs() as i64
}

fn http_status(code: StatusCode) -> VdrResult<ResponseType> {
    http_status_msg(code, code.to_string())
}

fn http_status_msg<T: std::fmt::Display>(code: StatusCode, msg: T) -> VdrResult<ResponseType> //Response<Body>, hyper::Error>
{
    Ok(ResponseType::Status(code, msg.to_string()))
}

async fn get_pool_genesis<T: Pool>(pool: &T) -> VdrResult<ResponseType> {
    let txns = pool.get_json_transactions()?;
    Ok(ResponseType::Genesis(txns.join("\n")))
}

fn get_pool_status(state: Rc<RefCell<AppState>>) -> VdrResult<ResponseType> {
    let opt_pool = &state.borrow().pool;
    let (status, mt_root, mt_size, nodes) = if let Some(pool) = opt_pool {
        let (mt_root, mt_size) = pool.get_merkle_tree_info();
        let nodes = pool.get_node_aliases();
        ("active", Some(mt_root), Some(mt_size), Some(nodes))
    } else {
        ("init", None, None, None)
    };
    let last_refresh = &state.borrow().last_refresh;
    let last_refresh = last_refresh.map(|tm| tm.elapsed().map(|d| d.as_secs()).ok());

    let result = json!({"status": status, "pool_mt_root": mt_root, "pool_mt_size": mt_size, "pool_nodes": nodes, "last_refresh": last_refresh});
    let result = serde_json::to_string(&result)
        .with_err_msg(VdrErrorKind::Unexpected, "Error serializing JSON")?;
    Ok(ResponseType::Json(result))
}

async fn get_attrib<T: Pool>(pool: &T, dest: &str, raw: &str) -> VdrResult<ResponseType> {
    let dest = DidValue::from_str(dest)?;
    let request = pool.get_request_builder().build_get_attrib_request(
        None,
        &dest,
        Some(raw.to_string()),
        None,
        None,
    )?;
    let result = perform_ledger_request(pool, &request).await?;
    Ok(result.into())
}

async fn get_nym<T: Pool>(pool: &T, nym: &str) -> VdrResult<ResponseType> {
    let nym = DidValue::from_str(nym)?;
    let request = pool
        .get_request_builder()
        .build_get_nym_request(None, &nym)?;
    let result = perform_ledger_request(pool, &request).await?;
    Ok(result.into())
}

async fn get_schema<T: Pool>(pool: &T, schema_id: &str) -> VdrResult<ResponseType> {
    let schema_id = SchemaId::from_str(schema_id)?;
    let request = pool
        .get_request_builder()
        .build_get_schema_request(None, &schema_id)?;
    let result = perform_ledger_request(pool, &request).await?;
    Ok(result.into())
}

async fn get_cred_def<T: Pool>(pool: &T, cred_def_id: &str) -> VdrResult<ResponseType> {
    let cred_def_id = CredentialDefinitionId::from_str(cred_def_id)?;
    let request = pool
        .get_request_builder()
        .build_get_cred_def_request(None, &cred_def_id)?;
    let result = perform_ledger_request(pool, &request).await?;
    Ok(result.into())
}

async fn get_revoc_reg_def<T: Pool>(pool: &T, revoc_reg_def_id: &str) -> VdrResult<ResponseType> {
    let revoc_reg_def_id = RevocationRegistryId::from_str(revoc_reg_def_id)?;
    let request = pool
        .get_request_builder()
        .build_get_revoc_reg_def_request(None, &revoc_reg_def_id)?;
    let result = perform_ledger_request(pool, &request).await?;
    Ok(result.into())
}

async fn get_revoc_reg<T: Pool>(pool: &T, revoc_reg_def_id: &str) -> VdrResult<ResponseType> {
    let revoc_reg_def_id = RevocationRegistryId::from_str(revoc_reg_def_id)?;
    let request = pool.get_request_builder().build_get_revoc_reg_request(
        None,
        &revoc_reg_def_id,
        timestamp_now(),
    )?;
    let result = perform_ledger_request(pool, &request).await?;
    Ok(result.into())
}

async fn get_revoc_reg_delta<T: Pool>(pool: &T, revoc_reg_def_id: &str) -> VdrResult<ResponseType> {
    let revoc_reg_def_id = RevocationRegistryId::from_str(revoc_reg_def_id)?;
    let request = pool
        .get_request_builder()
        .build_get_revoc_reg_delta_request(None, &revoc_reg_def_id, None, timestamp_now())?;
    let result = perform_ledger_request(pool, &request).await?;
    Ok(result.into())
}

/*
async fn test_get_validator_info<T: Pool>(pool: &T, pretty: bool) -> VdrResult<String> {
    let result = perform_get_validator_info(pool).await?;
    format_result(format_request_result(result, pretty))
}
*/

async fn get_taa<T: Pool>(pool: &T) -> VdrResult<ResponseType> {
    let request = pool
        .get_request_builder()
        .build_get_txn_author_agreement_request(None, None)?;
    let result = perform_ledger_request(pool, &request).await?;
    Ok(result.into())
}

async fn get_aml<T: Pool>(pool: &T) -> VdrResult<ResponseType> {
    let request = pool
        .get_request_builder()
        .build_get_acceptance_mechanisms_request(None, None, None)?;
    let result = perform_ledger_request(pool, &request).await?;
    Ok(result.into())
}

async fn get_auth_rule<T: Pool>(
    pool: &T,
    auth_type: Option<String>,
    auth_action: Option<String>,
    field: Option<String>,
) -> VdrResult<ResponseType> {
    let request = pool.get_request_builder().build_get_auth_rule_request(
        None,
        auth_type,
        auth_action,
        field,
        None,
        None,
    )?;
    let result = perform_ledger_request(pool, &request).await?;
    Ok(result.into())
}

async fn get_txn<T: Pool>(pool: &T, ledger: LedgerType, seq_no: i32) -> VdrResult<ResponseType> {
    let result = perform_get_txn(pool, ledger.to_id(), seq_no).await?;
    Ok(result.into())
}

async fn submit_request<T: Pool>(pool: &T, message: Vec<u8>) -> VdrResult<ResponseType> {
    let request = PreparedRequest::from_request_json(message)?;
    let result = perform_ledger_request(pool, &request).await?;
    Ok(result.into())
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
    let query = req.uri().query();
    let format = if query == Some("html") {
        ResponseFormat::Html
    } else if query == Some("raw") {
        ResponseFormat::Raw
    } else {
        if let Some(Ok(accept)) = req.headers().get("accept").map(|h| h.to_str()) {
            let accept = accept.to_ascii_lowercase();
            let html_pos = accept.find("text/html");
            let json_pos = accept.find("/json");
            match (html_pos, json_pos) {
                (Some(h), Some(j)) => {
                    if h < j {
                        ResponseFormat::Html
                    } else {
                        ResponseFormat::Raw
                    }
                }
                (Some(_), None) => ResponseFormat::Html,
                _ => ResponseFormat::Raw,
            }
        } else {
            ResponseFormat::Raw
        }
    };
    let fst = parts.next().unwrap_or_else(|| "".to_owned());
    let req_method = req.method();
    if (req_method, fst.is_empty()) == (&Method::GET, true) {
        return format_result(get_pool_status(state.clone()), format);
    }
    let opt_pool = &state.borrow().pool;
    let pool = match opt_pool {
        None => {
            return format_result(http_status(StatusCode::SERVICE_UNAVAILABLE), format);
        }
        Some(pool) => pool,
    };
    let result = match (req_method, fst.as_str()) {
        // (&Method::GET, "status") => test_get_validator_info(pool, pretty).await.make_response(),
        (&Method::GET, "submit") => http_status(StatusCode::METHOD_NOT_ALLOWED),
        (&Method::POST, "submit") => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
            let body = body_bytes.iter().cloned().collect::<Vec<u8>>();
            if !body.is_empty() {
                submit_request(pool, body).await
            } else {
                http_status(StatusCode::BAD_REQUEST)
            }
        }
        (&Method::GET, "genesis") => get_pool_genesis(pool).await,
        (&Method::GET, "taa") => get_taa(pool).await,
        (&Method::GET, "aml") => get_aml(pool).await,
        (&Method::GET, "attrib") => {
            if let (Some(dest), Some(attrib)) = (parts.next(), parts.next()) {
                // NOTE: 'endpoint' is currently the only supported attribute
                get_attrib(pool, &*dest, &*attrib).await
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
                    )
                    .await
                } else {
                    http_status(StatusCode::NOT_FOUND)
                }
            } else {
                get_auth_rule(pool, None, None, None).await // get all
            }
        }
        (&Method::GET, "cred_def") => {
            if let Some(cred_def_id) = parts.next() {
                get_cred_def(pool, &*cred_def_id).await
            } else {
                http_status(StatusCode::NOT_FOUND)
            }
        }
        (&Method::GET, "nym") => {
            if let Some(nym) = parts.next() {
                get_nym(pool, &*nym).await
            } else {
                http_status(StatusCode::NOT_FOUND)
            }
        }
        (&Method::GET, "rev_reg_def") => {
            if let Some(rev_reg_def_id) = parts.next() {
                get_revoc_reg_def(pool, &*rev_reg_def_id).await
            } else {
                http_status(StatusCode::NOT_FOUND)
            }
        }
        (&Method::GET, "rev_reg") => {
            if let Some(rev_reg_def_id) = parts.next() {
                get_revoc_reg(pool, &*rev_reg_def_id).await
            } else {
                http_status(StatusCode::NOT_FOUND)
            }
        }
        (&Method::GET, "rev_reg_delta") => {
            if let Some(rev_reg_def_id) = parts.next() {
                get_revoc_reg_delta(pool, &*rev_reg_def_id).await
            } else {
                http_status(StatusCode::NOT_FOUND)
            }
        }
        (&Method::GET, "schema") => {
            if let Some(schema_id) = parts.next() {
                get_schema(pool, &*schema_id).await
            } else {
                http_status(StatusCode::NOT_FOUND)
            }
        }
        (&Method::GET, "txn") => {
            if let (Some(ledger), Some(txn)) = (parts.next(), parts.next()) {
                if let (Ok(ledger), Ok(txn)) =
                    (LedgerType::try_from(ledger.as_str()), txn.parse::<i32>())
                {
                    get_txn(pool, ledger, txn).await
                } else {
                    http_status(StatusCode::NOT_FOUND)
                }
            } else {
                http_status(StatusCode::NOT_FOUND)
            }
        }
        (&Method::GET, _) => http_status(StatusCode::NOT_FOUND),
        _ => http_status(StatusCode::METHOD_NOT_ALLOWED),
    };
    format_result(result, format)
}
