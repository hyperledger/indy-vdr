extern crate env_logger;
extern crate indy_vdr;
extern crate log;

mod app;

use std::fs;
use std::net::IpAddr;
use std::process::exit;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Method, Request, Response, Server, StatusCode};
use hyper_unix_connector::UnixConnector;
use log::trace;

use indy_vdr::config::{LedgerResult, PoolFactory};
use indy_vdr::ledger::domain::txn::LedgerType;
use indy_vdr::pool::{
  perform_get_txn, perform_get_validator_info, perform_ledger_request, LocalPool, Pool,
  RequestResult, TimingResult,
};

fn main() {
  let config = app::load_config().unwrap_or_else(|err| {
    eprintln!("{}", err);
    exit(1);
  });

  env_logger::init();

  let mut rt = tokio::runtime::Builder::new()
    .enable_all()
    .basic_scheduler()
    .build()
    .expect("build runtime");

  let local = tokio::task::LocalSet::new();
  if let Err(err) = local.block_on(&mut rt, run(config)) {
    eprintln!("{}", err);
    exit(1);
  }
}

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

async fn handle_request<T: Pool>(
  req: Request<Body>,
  pool: T,
) -> Result<Response<Body>, hyper::Error> {
  match (req.method(), req.uri().path()) {
    (&Method::GET, "/") => {
      let msg = test_get_txn_single(1i32, &pool).await.unwrap();
      Ok(Response::new(Body::from(msg)))
    }
    (&Method::GET, "/status") => {
      let msg = test_get_validator_info(&pool).await.unwrap();
      Ok(Response::new(Body::from(msg)))
    }
    (&Method::GET, "/submit") => Ok(
      Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Body::default())
        .unwrap(),
    ),
    (&Method::POST, "/submit") => {
      let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
      let body = body_bytes.iter().cloned().collect::<Vec<u8>>();
      if !body.is_empty() {
        let (result, timing) = submit_request(&pool, body).await.unwrap();
        let mut response = Response::new(Body::from(result));
        response
          .headers_mut()
          .append("X-Requests", timing.parse().unwrap());
        Ok(response)
      } else {
        Ok(
          Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::default())
            .unwrap(),
        )
      }
    }
    (&Method::GET, "/genesis") => {
      let msg = get_genesis(&pool).await.unwrap();
      Ok(Response::new(Body::from(msg)))
    }
    (&Method::GET, "/taa") => {
      let msg = get_taa(&pool).await.unwrap();
      Ok(Response::new(Body::from(msg)))
    }
    (&Method::GET, "/aml") => {
      let msg = get_aml(&pool).await.unwrap();
      Ok(Response::new(Body::from(msg)))
    }
    (&Method::GET, _) => Ok(
      Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::default())
        .unwrap(),
    ),
    _ => Ok(
      Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Body::default())
        .unwrap(),
    ),
  }
}

async fn init_pool(genesis: String) -> LedgerResult<LocalPool> {
  let factory = PoolFactory::from_genesis_file(genesis.as_str())?;
  let pool = factory.create_local()?;
  Ok(pool)
}

async fn shutdown_signal() {
  tokio::signal::ctrl_c()
    .await
    .expect("failed to install CTRL+C signal handler");
}

async fn run(config: app::Config) -> Result<(), String> {
  // FIXME track status of pool and return server-not-ready when it hasn't initialized
  let pool = init_pool(config.genesis)
    .await
    .map_err(|err| format!("Error initializing pool: {}", err))?;

  // tokio::task::spawn_local(pool.refresh());

  let result = if let Some(socket) = config.socket {
    fs::remove_file(&socket)
      .map_err(|err| format!("Error removing socket: {}", err.to_string()))?;
    let uc: UnixConnector = tokio::net::UnixListener::bind(&socket)
      .map_err(|err| format!("Error binding UNIX socket: {}", err.to_string()))?
      .into();
    let svc = make_service_fn(move |_| {
      let pool = pool.clone();
      async move {
        let pool = pool.clone();
        Ok::<_, Error>(service_fn(move |req| handle_request(req, pool.to_owned())))
      }
    });
    let server = Server::builder(uc).executor(LocalExec).serve(svc);
    println!("Listening on {} ...", socket);
    server.with_graceful_shutdown(shutdown_signal()).await
  } else {
    let ip = config
      .host
      .unwrap()
      .parse::<IpAddr>()
      .map_err(|_| "Error parsing host IP")?;
    let addr = (ip, config.port.unwrap()).into();
    let builder = Server::try_bind(&addr)
      .map_err(|err| format!("Error binding TCP socket: {}", err.to_string()))?;
    let svc = make_service_fn(move |_| {
      let pool = pool.clone();
      async move {
        let pool = pool.clone();
        Ok::<_, Error>(service_fn(move |req| handle_request(req, pool.to_owned())))
      }
    });
    let server = builder.executor(LocalExec).serve(svc);
    println!("Listening on http://{} ...", addr);
    server.with_graceful_shutdown(shutdown_signal()).await
  };
  result.map_err(|err| format!("Server terminated: {}", err))
}

#[derive(Clone, Copy, Debug)]
struct LocalExec;

impl<F> hyper::rt::Executor<F> for LocalExec
where
  F: std::future::Future + 'static, // not requiring `Send`
{
  fn execute(&self, fut: F) {
    tokio::task::spawn_local(fut);
  }
}
