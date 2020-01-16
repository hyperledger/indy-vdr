extern crate env_logger;
extern crate indy_ledger_client;
extern crate log;

use std::cell::Cell;
use std::rc::Rc;

use log::trace;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Request, Response, Server};

use indy_ledger_client::config::{LedgerResult, PoolFactory};
use indy_ledger_client::services::pool::{perform_get_txn, LedgerType, Pool};

fn main() {
  env_logger::init();

  let mut rt = tokio::runtime::Builder::new()
    .enable_all()
    .basic_scheduler()
    .build()
    .expect("build runtime");

  let local = tokio::task::LocalSet::new();
  local.block_on(&mut rt, run()).unwrap();
  //local.block_on(&mut rt, test_get_txn()).unwrap();
}

async fn test_get_txn(seq_no: i32, pool: Pool) -> LedgerResult<String> {
  trace!("here");
  let result = perform_get_txn(&pool, LedgerType::DOMAIN, seq_no)
    .await
    .map_err(|err| err.to_string());
  let msg = if let Ok((msg, timing)) = result {
    format!("{}\n\n{:?}", msg, timing)
  } else {
    format!("{:?}", result)
  };
  Ok(msg)
}

async fn handle_request(
  _req: Request<Body>,
  seq_no: i32,
  pool: Pool,
) -> Result<Response<Body>, hyper::Error> {
  let msg = test_get_txn(seq_no, pool).await.unwrap();
  Ok::<_, Error>(Response::new(Body::from(msg)))
}

async fn run() -> LedgerResult<()> {
  let addr = ([127, 0, 0, 1], 3000).into();

  let factory = PoolFactory::from_genesis_file("genesis.txn")?;
  let pool = factory.create_pool()?;
  let count = Rc::new(Cell::new(1i32));

  let make_service = make_service_fn(move |_| {
    let pool = pool.clone();
    let count = count.clone();
    async move {
      Ok::<_, Error>(service_fn(move |req| {
        let prev = count.get();
        count.set(prev + 1);
        let seq_no = count.get();
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
