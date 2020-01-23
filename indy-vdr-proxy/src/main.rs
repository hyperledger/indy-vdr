extern crate env_logger;
extern crate indy_vdr;
extern crate log;

use std::cell::Cell;
use std::rc::Rc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Method, Request, Response, Server, StatusCode};
use log::trace;

use indy_vdr::config::{LedgerResult, PoolFactory};
use indy_vdr::ledger::domain::txn::LedgerType;
use indy_vdr::pool::{
  perform_get_txn, perform_get_txn_full, perform_get_validator_info, perform_ledger_request, Pool,
  RequestResult, TimingResult,
};

fn main() {
  env_logger::init();

  let mut rt = tokio::runtime::Builder::new()
    .enable_all()
    .basic_scheduler()
    .build()
    .expect("build runtime");

  let local = tokio::task::LocalSet::new();
  local.block_on(&mut rt, run()).unwrap();
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

async fn test_get_txn_full<T: Pool>(seq_no: i32, pool: &T) -> LedgerResult<String> {
  let result = perform_get_txn_full(pool, LedgerType::DOMAIN as i32, seq_no, None).await?;
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

async fn handle_request<T: Pool>(
  req: Request<Body>,
  seq_no: i32,
  pool: T,
) -> Result<Response<Body>, hyper::Error> {
  match (req.method(), req.uri().path()) {
    (&Method::GET, "/") => {
      let msg = test_get_txn_single(seq_no, &pool).await.unwrap();
      Ok::<_, Error>(Response::new(Body::from(msg)))
    }
    (&Method::GET, "/status") => {
      let msg = test_get_validator_info(&pool).await.unwrap();
      Ok::<_, Error>(Response::new(Body::from(msg)))
    }
    (&Method::GET, "/full") => {
      let msg = test_get_txn_full(seq_no, &pool).await.unwrap();
      Ok::<_, Error>(Response::new(Body::from(msg)))
    }
    (&Method::GET, "/genesis") => {
      let msg = get_genesis(&pool).await.unwrap();
      Ok::<_, Error>(Response::new(Body::from(msg)))
    }
    (&Method::GET, "/taa") => {
      let msg = get_taa(&pool).await.unwrap();
      Ok::<_, Error>(Response::new(Body::from(msg)))
    }
    (&Method::GET, "/aml") => {
      let msg = get_aml(&pool).await.unwrap();
      Ok::<_, Error>(Response::new(Body::from(msg)))
    }
    (&Method::GET, _) => {
      let mut not_found = Response::default();
      *not_found.status_mut() = StatusCode::NOT_FOUND;
      Ok(not_found)
    }
    _ => {
      let mut not_supported = Response::default();
      *not_supported.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
      Ok(not_supported)
    }
  }
}

async fn run() -> LedgerResult<()> {
  let addr = ([127, 0, 0, 1], 3000).into();

  let factory = PoolFactory::from_genesis_file("mainnet2.txn")?;
  let mut pool = factory.create_local()?;
  pool.refresh().await?;
  let count = Rc::new(Cell::new(1i32));

  let make_service = make_service_fn(move |_| {
    let pool = pool.clone();
    let count = count.clone();
    async move {
      Ok::<_, Error>(service_fn(move |req| {
        let seq_no = count.get();
        count.set(seq_no + 1);
        handle_request(req, seq_no, pool.to_owned())
      }))
    }
  });

  let server = Server::bind(&addr).executor(LocalExec).serve(make_service);

  println!("Listening on http://{}", addr);

  if let Err(e) = server.await {
    eprintln!("server error: {}", e);
  }

  Ok(())
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
