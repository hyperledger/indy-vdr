extern crate env_logger;
extern crate indy_vdr;
extern crate log;

mod app;
mod handlers;

use std::fs;
use std::net::IpAddr;
use std::process::exit;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use hyper_unix_connector::UnixConnector;

use indy_vdr::config::{LedgerResult, PoolFactory};
use indy_vdr::pool::LocalPool;

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
        Ok::<_, hyper::Error>(service_fn(move |req| {
          handlers::handle_request(req, pool.to_owned())
        }))
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
        Ok::<_, hyper::Error>(service_fn(move |req| {
          handlers::handle_request(req, pool.to_owned())
        }))
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
