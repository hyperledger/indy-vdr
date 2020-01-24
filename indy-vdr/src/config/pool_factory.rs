use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;
use std::{fs, io};

use serde_json;

use crate::common::error::prelude::*;
use crate::pool::{LocalPool, PoolConfig, ProtocolVersion, SharedPool, ZMQNetworker};

#[derive(Debug)]
pub struct PoolFactory {
    pub config: PoolConfig,
    pub transactions: Vec<String>,
}

impl PoolFactory {
    pub fn from_genesis_file(genesis_file: &str) -> LedgerResult<PoolFactory> {
        Self::from_genesis_path(&PathBuf::from(genesis_file))
    }

    pub fn from_genesis_path(genesis_path: &PathBuf) -> LedgerResult<PoolFactory> {
        let txns = _transactions_from_genesis(genesis_path)?;
        trace!("loaded transactions");
        if txns.len() == 0 {
            return Err((LedgerErrorKind::Config, "Empty genesis transaction file").into());
        }
        Ok(PoolFactory {
            config: PoolConfig::default(),
            transactions: txns,
        })
    }

    pub fn get_protocol_version(&self) -> ProtocolVersion {
        self.config.protocol_version
    }

    pub fn set_protocol_version(&mut self, version: ProtocolVersion) {
        self.config.protocol_version = version
    }

    pub fn get_transactions(&self) -> Vec<String> {
        self.transactions.clone()
    }

    pub fn create_local(&self) -> LedgerResult<LocalPool> {
        LocalPool::auto::<ZMQNetworker>(self.config, self.transactions.clone(), None)
    }

    pub fn create_shared(&self) -> LedgerResult<SharedPool> {
        SharedPool::auto::<ZMQNetworker>(self.config, self.transactions.clone(), None)
    }
}

fn _transactions_from_genesis(file_name: &PathBuf) -> LedgerResult<Vec<String>> {
    let mut result = vec![];

    let f = fs::File::open(file_name).with_input_err("Can't open genesis txn file")?;

    let reader = io::BufReader::new(&f);

    for line in reader.lines() {
        let line: String = line.with_input_err("Can't read from genesis txn file")?;

        if line.trim().is_empty() {
            continue;
        };

        // just validating, result is discarded
        let _: HashMap<String, serde_json::Value> =
            serde_json::from_str(&line).with_input_err("Genesis txn is malformed json")?;

        result.push(line);
    }

    Ok(result)
}
