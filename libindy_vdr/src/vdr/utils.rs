use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::{common::error::prelude::*, config::PoolConfig, pool::PoolTransactions};

use super::vdr::{PoolTxnAndConfig, GENESIS_FILENAME};

#[cfg(feature = "git")]
use git2::Repository;

#[cfg(feature = "git")]
pub fn clone_repo(repo_url: &str) -> VdrResult<Repository> {
    Repository::clone(repo_url, "github")
        .map_err(|_err| err_msg(VdrErrorKind::Unexpected, "Could not clone networks repo"))
}

pub fn folder_to_networks(
    path: PathBuf,
    genesis_filename: Option<&str>,
    default_config: Option<PoolConfig>,
) -> VdrResult<HashMap<String, PoolTxnAndConfig>> {
    let mut networks = HashMap::new();

    let genesis_filename = genesis_filename.or(Some(GENESIS_FILENAME)).unwrap();

    let entries = fs::read_dir(path).map_err(|err| {
        err_msg(
            VdrErrorKind::FileSystem(err),
            "Could not read local networks folder",
        )
    })?;

    for entry in entries {
        let entry = entry.unwrap();
        // filter hidden directories starting with "." and files
        if !entry.file_name().to_str().unwrap().starts_with(".")
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
                        PoolTransactions::from_json_file(sub_entry_path.join(genesis_filename))?,
                    ),
                    None => (
                        String::from(namespace.to_str().unwrap()),
                        PoolTransactions::from_json_file(entry.path().join(genesis_filename))?,
                    ),
                };
                let cfg = PoolTxnAndConfig {
                    txn: genesis_txns,
                    config: default_config.clone(),
                };
                networks.insert(ledger_prefix, cfg);
            }
        }
    }
    Ok(networks)
}
