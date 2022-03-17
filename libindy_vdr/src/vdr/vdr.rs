use git2::Repository;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::common::error::prelude::*;
use crate::ledger::responses::Endpoint;
use crate::pool::genesis::PoolTransactions;
use crate::pool::helpers::{perform_ledger_request, perform_refresh};
use crate::pool::{Pool, PoolBuilder, PreparedRequest, RequestResult, SharedPool, TimingResult};
use crate::resolver::did::DidUrl;
use crate::resolver::did_document::LEGACY_INDY_SERVICE;
use crate::resolver::resolver::*;
use crate::resolver::utils::*;
use crate::utils::did::DidValue;

const INDY_NETWORKS_GITHUB: &str = "https://github.com/IDunion/indy-did-networks";
const GENESIS_FILENAME: &str = "pool_transactions_genesis.json";

pub struct Vdr {
    pools: HashMap<String, SharedPool>,
}

impl Vdr {
    /// Expects a path to a folder with the following structure:
    /// <namespace>/<sub-namespace>/<genesis_file_name>
    /// Default for genesis_filename is pool_transactions_genesis.json
    /// Example: sovrin/staging/pool_transactions_genesis.json
    pub fn from_folder(path: PathBuf, genesis_filename: Option<&str>) -> VdrResult<Vdr> {
        debug!("Loading networks from local folder: {:?}", path);

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
                            PoolTransactions::from_json_file(
                                sub_entry_path.join(genesis_filename),
                            )?,
                        ),
                        None => (
                            String::from(namespace.to_str().unwrap()),
                            PoolTransactions::from_json_file(entry.path().join(genesis_filename))?,
                        ),
                    };
                    networks.insert(ledger_prefix, genesis_txns);
                }
            }
        }
        Vdr::new(networks)
    }

    /// Initialize VDR from a GitHub repo containing Indy network genesis files
    /// Default repo is https://github.com/IDunion/indy-did-networks
    pub fn from_github(repo_url: Option<&str>) -> VdrResult<Vdr> {
        let repo_url = repo_url.or(Some(INDY_NETWORKS_GITHUB)).unwrap();
        debug!("Obtaining network information from {}", repo_url);
        // Delete folder if it exists and reclone repo
        fs::remove_dir_all("github").ok();
        let repo = Repository::clone(INDY_NETWORKS_GITHUB, "github")
            .map_err(|_err| err_msg(VdrErrorKind::Unexpected, "Could not clone networks repo"))?;

        let path = repo.path().parent().unwrap().to_owned();

        Vdr::from_folder(path, None)
    }

    /// Create a new VDR instance from a map of namespaces and genesis transactions
    pub fn new(networks: HashMap<String, PoolTransactions>) -> VdrResult<Vdr> {
        let mut pools = HashMap::new();

        for (namespace, txns) in networks {
            let pool_builder = PoolBuilder::default().transactions(txns.to_owned())?;
            let pool = pool_builder.into_shared()?;
            pools.insert(namespace.to_string(), pool);
        }

        Ok(Vdr { pools })
    }

    /// Add a new ledger to the VDR
    pub fn add_ledger(&mut self, namespace: &str, pool: SharedPool) {
        self.pools.insert(String::from(namespace), pool);
    }

    /// Remove a ledger from the VDR
    pub fn remove_ledger(&mut self, namespace: &str) {
        self.pools.remove(namespace);
    }

    /// Get names of all ledgers
    pub fn get_ledgers(&self) -> VdrResult<Vec<String>> {
        Ok(self.pools.keys().cloned().collect::<Vec<String>>())
    }

    /// Get a validator pool reference
    pub fn get_pool(&self, namespace: &str) -> Option<&SharedPool> {
        self.borrow().pools.get(namespace)
    }
    /// Send a prepared request to a specific network
    pub async fn send_request(
        &self,
        namespace: &str,
        req: PreparedRequest,
    ) -> VdrResult<(RequestResult<String>, Option<TimingResult>)> {
        let pool = self
            .pools
            .get(namespace)
            .ok_or(err_msg(VdrErrorKind::Resolver, "Unkown namespace"))?;
        perform_ledger_request(pool, &req).await
    }

    /// Dereference a DID Url and return a serialized `DereferencingResult`
    pub async fn dereference(&self, did_url: &str) -> VdrResult<String> {
        debug!("VDR: Dereference DID Url {}", did_url);
        let did_url = DidUrl::from_str(did_url)?;
        let (data, metadata) = self._resolve(&did_url).await?;

        let content = match data {
            Result::Content(c) => Some(c),
            _ => None,
        };

        let md = if let Metadata::ContentMetadata(md) = metadata {
            Some(md)
        } else {
            None
        };

        let result = DereferencingResult {
            dereferencing_metadata: None,
            content_stream: content,
            content_metadata: md,
        };

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    /// Resolve a DID and return a serialized `ResolutionResult`
    pub async fn resolve(&self, did: &str) -> VdrResult<String> {
        debug!("VDR Resolve DID {}", did);
        let did = DidUrl::from_str(did)?;
        let (data, metadata) = self._resolve(&did).await?;

        let diddoc = match data {
            Result::DidDocument(mut doc) => {
                // Try to find legacy endpoint if diddoc_content is none
                if doc.diddoc_content.is_none() {
                    let pool = self
                        .pools
                        .get(&did.namespace)
                        .ok_or(err_msg(VdrErrorKind::Resolver, "Unkown namespace"))?;
                    doc.endpoint = fetch_legacy_endpoint(pool, &did.id).await.ok();
                }
                Some(doc.to_value()?)
            }
            _ => None,
        };

        let md = if let Metadata::DidDocumentMetadata(md) = metadata {
            Some(md)
        } else {
            None
        };

        let result = ResolutionResult {
            did_resolution_metadata: None,
            did_document: diddoc,
            did_document_metadata: md,
        };

        Ok(serde_json::to_string_pretty(&result).unwrap())
    }

    // Internal method to resolve and dereference
    async fn _resolve(&self, did_url: &DidUrl) -> VdrResult<(Result, Metadata)> {
        let pool = self
            .pools
            .get(&did_url.namespace)
            .ok_or(err_msg(VdrErrorKind::Resolver, "Unkown namespace"))?;

        let builder = pool.get_request_builder();
        let request = build_request(&did_url, &builder)?;

        let ledger_data = handle_request(pool, &request).await?;
        let txn_type = &request.txn_type.as_str();
        let result = handle_resolution_result(&did_url, &ledger_data, txn_type)?;

        Ok(result)
    }

    /// Refresh validator pools for all networks
    pub async fn refresh_all(&mut self) -> VdrResult<()> {
        let keys: Vec<String> = self.pools.keys().cloned().collect();

        for k in keys.iter() {
            let pool = self.pools.get_mut(k).unwrap();
            let (txns, _) = perform_refresh(pool).await?;
            let p = if let Some(txns) = txns {
                let builder = {
                    let mut current_txns =
                        PoolTransactions::from_json_transactions(pool.get_json_transactions()?)
                            .unwrap();
                    current_txns.extend_from_json(&txns)?;
                    PoolBuilder::default().transactions(current_txns.clone())?
                };
                builder.into_shared()?
            } else {
                pool.to_owned()
            };
            *pool = p;
        }
        Ok(())
    }

    /// Refresh the validator pool of a particular network
    pub async fn refresh(&mut self, namespace: &str) -> VdrResult<()> {
        let pool = self
            .pools
            .get_mut(namespace)
            .ok_or("Unkown namespace")
            .map_err(|err| err_msg(VdrErrorKind::Resolver, err))?;

        let (txns, _) = perform_refresh(pool).await?;
        let p = if let Some(txns) = txns {
            let builder = {
                let mut current_txns =
                    PoolTransactions::from_json_transactions(pool.get_json_transactions()?)
                        .unwrap();
                current_txns.extend_from_json(&txns)?;
                PoolBuilder::default().transactions(current_txns.clone())?
            };
            builder.into_shared()?
        } else {
            pool.to_owned()
        };
        *pool = p;

        Ok(())
    }
}

async fn handle_request(pool: &SharedPool, request: &PreparedRequest) -> VdrResult<String> {
    let (result, _timing) = request_transaction(pool, &request).await?;
    match result {
        RequestResult::Reply(data) => Ok(data),
        RequestResult::Failed(error) => Err(error),
    }
}

async fn request_transaction(
    pool: &SharedPool,
    request: &PreparedRequest,
) -> VdrResult<(RequestResult<String>, Option<TimingResult>)> {
    perform_ledger_request(pool, &request).await
}

/// Fetch legacy service endpoint using ATTRIB tx
async fn fetch_legacy_endpoint(pool: &SharedPool, did: &DidValue) -> VdrResult<Endpoint> {
    let builder = pool.get_request_builder();
    let request = builder.build_get_attrib_request(
        None,
        did,
        Some(String::from(LEGACY_INDY_SERVICE)),
        None,
        None,
    )?;
    let ledger_data = handle_request(pool, &request).await?;
    let endpoint_data = parse_ledger_data(&ledger_data)?;
    let endpoint_data: Endpoint = serde_json::from_str(endpoint_data.as_str().unwrap())
        .map_err(|_| err_msg(VdrErrorKind::Resolver, "Could not parse endpoint data"))?;
    Ok(endpoint_data)
}
