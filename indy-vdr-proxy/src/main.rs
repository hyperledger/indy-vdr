#![allow(clippy::await_holding_refcell_ref)] // using a single-threaded executor

#[macro_use]
extern crate serde_json;

mod app;
mod handlers;
mod utils;

use std::cell::RefCell;
use std::collections::HashMap;
#[cfg(unix)]
use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;
use std::process::exit;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use futures_util::future::FutureExt;
use git2::Repository;

#[cfg(feature = "fetch")]
use hyper::body::Buf;
use hyper::service::{make_service_fn, service_fn};
#[cfg(feature = "fetch")]
use hyper::Client;
use hyper::Server;
#[cfg(feature = "fetch")]
use hyper_tls::HttpsConnector;

#[cfg(unix)]
use hyper_unix_connector::UnixConnector;

use tokio::select;
#[cfg(unix)]
use tokio::signal::unix::SignalKind;

use indy_vdr::common::error::prelude::*;
use indy_vdr::pool::{helpers::perform_refresh, LocalPool, PoolBuilder, PoolTransactions};

use crate::utils::{
    init_pool_state_from_folder_structure, AppState, PoolState, INDY_NETWORKS_GITHUB,
};

fn main() {
    let config = app::load_config().unwrap_or_else(|err| {
        eprintln!("{}", err);
        exit(1);
    });

    env_logger::init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build runtime");

    let local = tokio::task::LocalSet::new();
    if let Err(err) = local.block_on(&rt, init_server(config)) {
        eprintln!("{}", err);
        exit(1);
    }
}

#[cfg(feature = "fetch")]
async fn fetch_transactions(genesis: String) -> VdrResult<PoolTransactions> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let mut res = client
        .get(genesis.parse().with_err_msg(
            VdrErrorKind::Config,
            format!("Error parsing genesis URL: {}", genesis),
        )?)
        .await
        .with_err_msg(VdrErrorKind::Config, "Error fetching genesis transactions")?;
    if res.status() != 200 {
        return Err(err_msg(
            VdrErrorKind::Config,
            format!(
                "Unexpected HTTP status for genesis transactions: {}",
                res.status()
            ),
        ));
    };
    let mut buf = hyper::body::aggregate(res.body_mut())
        .await
        .with_err_msg(VdrErrorKind::Config, "Error receiving genesis transactions")?;
    let body = buf.copy_to_bytes(buf.remaining());
    let txns = String::from_utf8_lossy(&body);
    PoolTransactions::from_json(&txns)
}

#[cfg(not(feature = "fetch"))]
async fn fetch_transactions(_genesis: String) -> VdrResult<PoolTransactions> {
    Err(err_msg(
        VdrErrorKind::Config,
        "This application is not compiled with HTTP(S) request support",
    ))
}

async fn init_app_state(
    genesis: Option<String>,
    namespace: String,
    is_multiple: bool,
) -> VdrResult<AppState> {
    let mut pool_states: HashMap<String, PoolState> = HashMap::new();

    let state = if !is_multiple {
        let genesis = genesis.unwrap_or_else(|| String::from("genesis.txn"));
        let transactions = if genesis.starts_with("http:") || genesis.starts_with("https:") {
            fetch_transactions(genesis).await?
        } else {
            PoolTransactions::from_json_file(genesis.as_str())?
        };
        let pool_state = PoolState {
            pool: None,
            last_refresh: None,
            transactions,
        };
        pool_states.insert(namespace, pool_state);
        AppState {
            is_multiple,
            pool_states,
        }
    } else {
        let genesis = genesis.unwrap_or_else(|| String::from(INDY_NETWORKS_GITHUB));
        let pool_states = if genesis.starts_with("https:") {
            let repo_url = genesis;
            let mut just_cloned = false;
            let repo =
                git2::Repository::discover("github").or_else(|_| -> VdrResult<Repository> {
                    just_cloned = true;
                    Repository::clone(&repo_url, "github").map_err(|_err| {
                        err_msg(VdrErrorKind::Unexpected, "Could not clone networks repo")
                    })
                })?;

            // Fetch remote if not cloned just now
            if !just_cloned {
                let mut origin_remote = repo.find_remote("origin").map_err(|_err| {
                    err_msg(
                        VdrErrorKind::Unexpected,
                        "Networks repo has no remote origin",
                    )
                })?;

                origin_remote.fetch(&["main"], None, None).map_err(|_err| {
                    err_msg(
                        VdrErrorKind::Unexpected,
                        "Could not fetch from remote networks repo",
                    )
                })?;
            }

            let path = repo.path().parent().unwrap().to_owned();

            init_pool_state_from_folder_structure(path)?
        } else {
            init_pool_state_from_folder_structure(PathBuf::from(genesis))?
        };
        AppState {
            is_multiple,
            pool_states,
        }
    };
    Ok(state)
}

async fn run_pools(state: Rc<RefCell<AppState>>, init_refresh: bool, interval_refresh: u32) {
    let mut pool_states = HashMap::new();

    for (namespace, pool_state) in &state.clone().borrow().pool_states {
        let pool_state = match create_pool(state.clone(), namespace.as_str(), init_refresh).await {
            Ok(pool) => {
                let pool = Some(pool.clone());
                PoolState {
                    pool: pool.clone(),
                    last_refresh: pool_state.last_refresh,
                    transactions: pool_state.transactions.clone(),
                }
            }
            Err(err) => {
                eprintln!("Error initializing pool {} with error : {}", namespace, err);
                PoolState {
                    pool: None,
                    last_refresh: pool_state.last_refresh,
                    transactions: pool_state.transactions.clone(),
                }
            }
        };

        pool_states.insert(namespace.to_owned(), pool_state);
    }

    state.borrow_mut().pool_states = pool_states;

    let shutdown = shutdown_signal().fuse().shared();
    if interval_refresh > 0 {
        loop {
            select! {
                refresh_result = refresh_pools(state.clone(), interval_refresh) => {
                    match refresh_result {
                        Ok(upd_pool_states) => {
                            state.borrow_mut().pool_states = upd_pool_states;
                            log::info!("Refreshed validator pools");
                        },
                        Err(err) => {
                            log::error!("Error refreshing validator pool: {}", err);
                        }
                    }
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
        _ = term.recv() => {}
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

async fn create_pool(
    state: Rc<RefCell<AppState>>,
    namespace: &str,
    refresh: bool,
) -> VdrResult<LocalPool> {
    let pool_states = &state.borrow().pool_states;
    let pool_state = pool_states.get(namespace).unwrap();
    let builder = PoolBuilder::default().transactions(pool_state.transactions.clone())?;
    let pool = builder.into_local()?;
    let refresh_pool = if refresh {
        refresh_pool(state.clone(), namespace, &pool, 0).await?
    } else {
        None
    };
    Ok(refresh_pool.unwrap_or(pool))
}

async fn refresh_pools(
    state: Rc<RefCell<AppState>>,
    // pool_states: HashMap<String, PoolState>,
    delay_mins: u32,
) -> VdrResult<HashMap<String, PoolState>> {
    let mut upd_pool_states = HashMap::new();
    let pool_states = &state.borrow().pool_states;
    for (namespace, pool_state) in pool_states {
        if let Some(pool) = &pool_state.pool {
            let upd_pool = match refresh_pool(state.clone(), namespace, pool, delay_mins).await {
                Ok(p) => p,
                Err(err) => {
                    eprintln!(
                        "Error refreshing validator pool {} with error {}",
                        namespace, err
                    );
                    None
                }
            };
            let upd_pool_state = PoolState {
                pool: upd_pool.or_else(|| Some(pool.clone())),
                last_refresh: Some(SystemTime::now()),
                transactions: pool_state.transactions.clone(),
            };

            upd_pool_states.insert(namespace.to_owned(), upd_pool_state);
        }
    }

    Ok(upd_pool_states)
}

async fn refresh_pool(
    state: Rc<RefCell<AppState>>,
    namespace: &str,
    pool: &LocalPool,
    delay_mins: u32,
) -> VdrResult<Option<LocalPool>> {
    let n_pools = state.borrow().pool_states.len() as u32;
    if delay_mins > 0 {
        tokio::time::sleep(Duration::from_secs((delay_mins * 60 / n_pools) as u64)).await
    }

    let (txns, _timing) = perform_refresh(pool).await?;

    let cloned_state = state.clone();
    let pool_states = &cloned_state.borrow().pool_states;
    let pool_state = pool_states.get(namespace).unwrap();

    let pool_txns = &mut pool_state.transactions.to_owned();

    if let Some(txns) = txns {
        let builder = {
            pool_txns.extend_from_json(&txns)?;
            PoolBuilder::default().transactions(pool_txns.clone())?
        };
        Ok(Some(builder.into_local()?))
    } else {
        Ok(None)
    }
}

async fn init_server(config: app::Config) -> Result<(), String> {
    let state = Rc::new(RefCell::new(
        init_app_state(
            config.genesis.clone(),
            config.namespace.clone(),
            config.is_multiple,
        )
        .await
        .map_err(|err| format!("Error loading config: {}", err))?,
    ));

    #[cfg(unix)]
    if let Some(socket) = &config.socket {
        fs::remove_file(socket).map_err(|err| format!("Error removing socket: {}", err))?;
        let uc: UnixConnector = tokio::net::UnixListener::bind(socket)
            .map_err(|err| format!("Error binding UNIX socket: {}", err))?
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
    let builder =
        Server::try_bind(&addr).map_err(|err| format!("Error binding TCP socket: {}", err))?;
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
    let until_done = run_pools(state.clone(), config.init_refresh, config.interval_refresh);
    let svc = make_service_fn(move |_| {
        let state = state.clone();
        async move {
            let state = state.clone();
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handlers::handle_request(req, state.to_owned())
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
