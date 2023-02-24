use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use indy_vdr::common::error::prelude::*;
use indy_vdr::pool::{LocalPool, PoolTransactions};

pub const INDY_NETWORKS_GITHUB: &str = "https://github.com/IDunion/indy-did-networks";
pub const GENESIS_FILENAME: &str = "pool_transactions_genesis.json";

pub struct PoolState {
    pub pool: Option<LocalPool>,
    pub last_refresh: Option<SystemTime>,
    pub transactions: PoolTransactions,
}

pub struct AppState {
    pub is_multiple: bool,
    pub pool_states: HashMap<String, PoolState>,
}

pub fn init_pool_state_from_folder_structure(
    path: PathBuf,
) -> VdrResult<HashMap<String, PoolState>> {
    let mut networks = HashMap::new();

    let entries = fs::read_dir(path).map_err(|err| {
        err_msg(
            VdrErrorKind::FileSystem(err),
            "Could not read local networks folder",
        )
    })?;

    for entry in entries {
        let entry = entry.unwrap();
        // filter hidden directories starting with "." and files
        if !entry.file_name().to_str().unwrap().starts_with('.')
            && entry.metadata().unwrap().is_dir()
        {
            let namespace = entry.path().file_name().unwrap().to_owned();
            let sub_entries = fs::read_dir(entry.path()).unwrap();
            for sub_entry in sub_entries {
                let sub_entry_path = sub_entry.unwrap().path();
                let sub_namespace = if sub_entry_path.is_dir() {
                    sub_entry_path.file_name()
                } else {
                    None
                };
                let (ledger_prefix, genesis_txns) = match sub_namespace {
                    Some(sub_namespace) => (
                        format!(
                            "{}:{}",
                            namespace.to_str().unwrap(),
                            sub_namespace.to_str().unwrap()
                        ),
                        PoolTransactions::from_json_file(sub_entry_path.join(GENESIS_FILENAME))?,
                    ),
                    None => (
                        String::from(namespace.to_str().unwrap()),
                        PoolTransactions::from_json_file(entry.path().join(GENESIS_FILENAME))?,
                    ),
                };
                let pool_state = PoolState {
                    pool: None,
                    last_refresh: None,
                    transactions: genesis_txns,
                };
                networks.insert(ledger_prefix, pool_state);
            }
        }
    }
    Ok(networks)
}
