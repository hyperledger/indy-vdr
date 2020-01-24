extern crate env_logger;
extern crate indy_vdr;
extern crate log;

use hyper::{Body, Method, Request, Response, StatusCode};
use log::trace;

use indy_vdr::config::{LedgerError, LedgerErrorKind, LedgerResult};
use indy_vdr::ledger::domain::txn::LedgerType;
use indy_vdr::pool::{
    perform_get_txn, perform_get_validator_info, perform_ledger_request, Pool, RequestResult,
    TimingResult,
};

fn format_request_result<T: std::fmt::Display>(
    (result, timing): (RequestResult<T>, Option<TimingResult>),
) -> LedgerResult<(T, TimingResult)> {
    match result {
        RequestResult::Reply(message) => {
            trace!("Got request response {} {:?}", &message, timing);
            Ok((message, timing.unwrap()))
        }
        RequestResult::Failed(err) => {
            trace!("No consensus {:?}", timing);
            Err(err)
        }
    }
}

fn format_result<T: std::fmt::Debug>(result: LedgerResult<(String, T)>) -> LedgerResult<String> {
    Ok(match result {
        Ok((msg, timing)) => format!("{}\n\n{:?}", msg, timing),
        Err(err) => err.to_string(),
    })
}

async fn test_get_txn_single<T: Pool>(seq_no: i32, pool: &T) -> LedgerResult<String> {
    let result = perform_get_txn(pool, LedgerType::DOMAIN as i32, seq_no).await?;
    format_result(format_request_result(result))
}

async fn get_genesis<T: Pool>(pool: &T) -> LedgerResult<String> {
    let txns = pool.get_transactions();
    Ok(txns.join("\n"))
}

async fn test_get_validator_info<T: Pool>(pool: &T) -> LedgerResult<String> {
    let result = perform_get_validator_info(pool).await?;
    format_result(format_request_result(result))
}

async fn get_taa<T: Pool>(pool: &T) -> LedgerResult<String> {
    let request = pool
        .get_request_builder()
        .build_get_txn_author_agreement_request(None, None)?;
    let result = perform_ledger_request(pool, request, None).await?;
    format_result(format_request_result(result))
}

async fn get_aml<T: Pool>(pool: &T) -> LedgerResult<String> {
    let request = pool
        .get_request_builder()
        .build_get_acceptance_mechanisms_request(None, None, None)?;
    let result = perform_ledger_request(pool, request, None).await?;
    format_result(format_request_result(result))
}

async fn submit_request<T: Pool>(pool: &T, message: Vec<u8>) -> LedgerResult<(String, String)> {
    let (request, target) = pool.get_request_builder().parse_inbound_request(&message)?;
    let result = perform_ledger_request(pool, request, target).await?;
    let (response, timing) = format_request_result(result)?;
    Ok((response, format!("{:?}", timing)))
}

fn format_ledger_error(err: LedgerError) -> Result<Response<Body>, hyper::Error> {
    let msg = err.to_string();
    let (errcode, msg) = match err.kind() {
        LedgerErrorKind::PoolRequestFailed(failed) => (StatusCode::BAD_REQUEST, failed),
        LedgerErrorKind::Input => (StatusCode::BAD_REQUEST, msg),
        LedgerErrorKind::PoolTimeout => (StatusCode::GATEWAY_TIMEOUT, msg),
        LedgerErrorKind::PoolNoConsensus => (StatusCode::CONFLICT, msg),
        // FIXME - UNAUTHORIZED error when BadRequest msg points to a missing signature
        _ => (StatusCode::INTERNAL_SERVER_ERROR, msg),
    };
    err_status_msg(errcode, msg)
}

trait HandleLedgerError {
    fn make_response(self) -> Result<Response<Body>, hyper::Error>;
}

impl<T> HandleLedgerError for LedgerResult<T>
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

fn err_status(code: StatusCode) -> Result<Response<Body>, hyper::Error> {
    err_status_msg(code, "")
}

fn err_status_msg<M>(code: StatusCode, msg: M) -> Result<Response<Body>, hyper::Error>
where
    Body: From<M>,
{
    Ok(Response::builder()
        .status(code)
        .body(Body::from(msg))
        .unwrap())
}

pub async fn handle_request<T: Pool>(
    req: Request<Body>,
    pool: T,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => test_get_txn_single(1i32, &pool).await.make_response(),
        (&Method::GET, "/status") => test_get_validator_info(&pool).await.make_response(),
        (&Method::GET, "/submit") => err_status(StatusCode::METHOD_NOT_ALLOWED),
        (&Method::POST, "/submit") => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
            let body = body_bytes.iter().cloned().collect::<Vec<u8>>();
            if !body.is_empty() {
                match submit_request(&pool, body).await {
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
                err_status(StatusCode::BAD_REQUEST)
            }
        }
        (&Method::GET, "/genesis") => get_genesis(&pool).await.make_response(),
        (&Method::GET, "/taa") => get_taa(&pool).await.make_response(),
        (&Method::GET, "/aml") => get_aml(&pool).await.make_response(),
        (&Method::GET, _) => err_status(StatusCode::NOT_FOUND),
        _ => err_status(StatusCode::METHOD_NOT_ALLOWED),
    }
}
