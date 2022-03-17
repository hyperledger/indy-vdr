#[macro_use]
extern crate serde_json;

mod app;
mod handlers;

use std::cell::RefCell;
#[cfg(unix)]
use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;
use std::process::exit;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use futures_util::FutureExt;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

#[cfg(unix)]
use hyper_unix_connector::UnixConnector;

use indy_vdr::pool::SharedPool;
use tokio::select;
#[cfg(unix)]
use tokio::signal::unix::SignalKind;

use indy_vdr::common::error::prelude::*;
use indy_vdr::vdr::Vdr;

fn main() {
    let config = app::load_config().unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });

    env_logger::init();

    let mut rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build runtime");

    let local = tokio::task::LocalSet::new();
    if let Err(err) = local.block_on(&mut rt, init_server(config)) {
        eprintln!("{}", err);
        exit(1);
    }
}

pub struct AppState {
    vdr: Vdr,
    last_refresh: Option<SystemTime>,
}

async fn init_app_state(genesis: Option<String>) -> VdrResult<AppState> {
    let vdr = match genesis {
        Some(genesis) => {
            if genesis.starts_with("http:") || genesis.starts_with("https:") {
                Vdr::from_github(Some(genesis.as_str()))?
            } else {
                Vdr::from_folder(PathBuf::from(genesis), None)?
            }
        }
        None => Vdr::from_github(None)?,
    };

    let state = AppState {
        vdr,
        last_refresh: None,
    };
    Ok(state)
}

async fn run_pool(state: Rc<RefCell<AppState>>, init_refresh: bool, interval_refresh: u32) {
    if init_refresh {
        state.borrow_mut().last_refresh.replace(SystemTime::now());
        if let Err(err) = state.borrow_mut().vdr.refresh_all().await {
            eprintln!("Could not refresh validator pool with err: {:?}", err);
        }
    }
    let shutdown = shutdown_signal().fuse().shared();
    if interval_refresh > 0 {
        loop {
            select! {

                _ = refresh_pools(state.clone(), interval_refresh) => {

                            log::info!("Refreshed validator pool");

                }
                _ = shutdown.clone() => {
                    println!("Shutting down");
                    break;
                }
            }
        }
    } else {
        shutdown.await
    }
}

#[cfg(unix)]
async fn shutdown_signal() {
    let mut term = tokio::signal::unix::signal(SignalKind::terminate())
        .expect("failed to install SIGTERM handler");
    select! {
        _ = term.recv() => {
            ()
        }
        ctlc = tokio::signal::ctrl_c() => {
            ctlc.expect("failed to install Ctrl-C handler")
        }
    }
}

#[cfg(not(unix))]
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl-C handler")
}

async fn refresh_pools(state: Rc<RefCell<AppState>>, delay_mins: u32) {
    if delay_mins > 0 {
        tokio::time::sleep(Duration::from_secs((delay_mins * 60) as u64)).await
    }

    state.borrow_mut().last_refresh.replace(SystemTime::now());

    if let Err(err) = state.borrow_mut().vdr.refresh_all().await {
        eprintln!("Could not refresh validator pool with err: {:?}", err);
    }
}

async fn init_server(config: app::Config) -> Result<(), String> {
    let state = Rc::new(RefCell::new(
        init_app_state(config.genesis.clone())
            .await
            .map_err(|err| format!("Error loading config: {}", err))?,
    ));

    #[cfg(unix)]
    if let Some(socket) = &config.socket {
        fs::remove_file(socket)
            .map_err(|err| format!("Error removing socket: {}", err.to_string()))?;
        let uc: UnixConnector = tokio::net::UnixListener::bind(socket)
            .map_err(|err| format!("Error binding UNIX socket: {}", err.to_string()))?
            .into();
        return run_server(
            Server::builder(uc),
            state,
            format!("socket {}", socket),
            config,
        )
        .await;
    }

    let ip = config
        .host
        .as_ref()
        .unwrap()
        .parse::<IpAddr>()
        .map_err(|_| "Error parsing host IP")?;
    let addr = (ip, config.port.unwrap()).into();
    let builder = Server::try_bind(&addr)
        .map_err(|err| format!("Error binding TCP socket: {}", err.to_string()))?;
    run_server(builder, state, format!("http://{}", addr), config).await
}

async fn run_server<I>(
    builder: hyper::server::Builder<I>,
    state: Rc<RefCell<AppState>>,
    address: String,
    config: app::Config,
) -> Result<(), String>
where
    I: hyper::server::accept::Accept + 'static,
    I::Conn: tokio::io::AsyncRead + tokio::io::AsyncWrite + Send + Unpin,
    I::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let until_done = run_pool(state.clone(), config.init_refresh, config.interval_refresh);
    let svc = make_service_fn(move |_| {
        let state = state.clone();
        async move {
            let state = state.clone();
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handlers::handle_request::<SharedPool>(req, state.to_owned())
            }))
        }
    });
    let server = builder.executor(LocalExec).serve(svc);
    println!("Listening on {} ...", address);

    server
        .with_graceful_shutdown(until_done)
        .await
        .map_err(|err| format!("Server terminated: {}", err))
}

#[derive(Clone, Copy, Debug)]
struct LocalExec;

impl<F> hyper::rt::Executor<F> for LocalExec
where
    F: std::future::Future + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn_local(fut);
    }
}
