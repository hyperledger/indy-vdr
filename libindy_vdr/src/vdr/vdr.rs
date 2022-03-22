#[cfg(feature = "git")]
use git2::Repository;

use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::common::error::prelude::*;
use crate::config::PoolConfig;
use crate::ledger::RequestBuilder;
use crate::pool::genesis::PoolTransactions;
use crate::pool::helpers::{perform_ledger_request, perform_refresh};
use crate::pool::{
    Pool, PoolBuilder, PoolRunner, PreparedRequest, RequestResult, SharedPool, TimingResult,
};
use crate::resolver::did::DidUrl;
use crate::resolver::types::*;
use crate::resolver::utils::*;

use super::utils::*;

#[cfg(feature = "git")]
const INDY_NETWORKS_GITHUB: &str = "https://github.com/IDunion/indy-did-networks";
pub const GENESIS_FILENAME: &str = "pool_transactions_genesis.json";

#[derive(Clone, Debug)]
pub struct PoolTxnAndConfig {
    pub txn: PoolTransactions,
    pub config: Option<PoolConfig>,
}

pub struct Vdr {
    pools: HashMap<String, SharedPool>,
}

impl Vdr {
    /// Expects a path to a folder with the following structure:
    /// <namespace>/<sub-namespace>/<genesis_file_name>
    /// Default for genesis_filename is pool_transactions_genesis.json
    /// Example: sovrin/staging/pool_transactions_genesis.json
    pub fn from_folder(
        path: PathBuf,
        genesis_filename: Option<&str>,
        default_config: Option<PoolConfig>,
    ) -> VdrResult<Vdr> {
        debug!("Loading networks from local folder: {:?}", path);

        let networks = folder_to_networks(path, genesis_filename, default_config)?;

        Vdr::new(networks)
    }

    #[cfg(feature = "git")]
    /// Initialize VDR from a GitHub repo containing Indy network genesis files
    /// Default repo is https://github.com/IDunion/indy-did-networks
    pub fn from_github(
        repo_url: Option<&str>,
        force_clone: Option<bool>,
        default_config: Option<PoolConfig>,
    ) -> VdrResult<Vdr> {
        let repo_url = repo_url.or(Some(INDY_NETWORKS_GITHUB)).unwrap();
        debug!("Obtaining network information from {}", repo_url);

        let mut cloned = false;

        let repo = if force_clone.is_some() && force_clone.unwrap() {
            cloned = true;
            clone_repo(repo_url)?
        } else {
            git2::Repository::discover("github").or_else(|_| -> VdrResult<Repository> {
                cloned = true;
                clone_repo(repo_url)
            })?
        };

        // Fetch remote if not cloned just now
        if !cloned {
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

        Vdr::from_folder(path, None, default_config)
    }

    /// Create a new VDR instance from a map of namespaces and genesis transactions
    pub fn new(networks: HashMap<String, PoolTxnAndConfig>) -> VdrResult<Vdr> {
        let mut pools = HashMap::new();

        for (namespace, cfg) in networks {
            let txn = cfg.txn;
            let config = cfg.config;

            let pool_builder = if config.is_some() {
                PoolBuilder::new(config.unwrap(), None, None).transactions(txn.to_owned())?
            } else {
                PoolBuilder::default().transactions(txn.to_owned())?
            };
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
                // Try to find legacy endpoint using a GET_ATTRIB txn if diddoc_content is none
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
        let txn_type = request.txn_type.as_str();
        let namespace = did_url.namespace.clone();
        let result = handle_resolution_result(namespace.as_str(), &ledger_data, txn_type)?;

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

pub struct RunnerVdr {
    pools: HashMap<String, PoolRunner>,
}

impl RunnerVdr {
    /// Expects a path to a folder with the following structure:
    /// <namespace>/<sub-namespace>/<genesis_file_name>
    /// Default for genesis_filename is pool_transactions_genesis.json
    /// Example: sovrin/staging/pool_transactions_genesis.json
    pub fn from_folder(
        path: PathBuf,
        genesis_filename: Option<&str>,
        default_config: Option<PoolConfig>,
    ) -> VdrResult<RunnerVdr> {
        debug!("Loading networks from local folder: {:?}", path);
        let networks = folder_to_networks(path, genesis_filename, default_config)?;
        RunnerVdr::new(networks)
    }

    #[cfg(feature = "git")]
    /// Initialize VDR from a GitHub repo containing Indy network genesis files
    /// Default repo is https://github.com/IDunion/indy-did-networks
    pub fn from_github(
        repo_url: Option<&str>,
        force_clone: Option<bool>,
        default_config: Option<PoolConfig>,
    ) -> VdrResult<RunnerVdr> {
        let repo_url = repo_url.or(Some(INDY_NETWORKS_GITHUB)).unwrap();
        debug!("Obtaining network information from {}", repo_url);
        let mut cloned = false;

        let repo = if force_clone.is_some() && force_clone.unwrap() {
            cloned = true;
            clone_repo(repo_url)?
        } else {
            git2::Repository::discover("github").or_else(|_| -> VdrResult<Repository> {
                cloned = true;
                clone_repo(repo_url)
            })?
        };

        // Fetch remote if not cloned just now
        if !cloned {
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

        RunnerVdr::from_folder(path, None, default_config)
    }

    /// Create a new VDR instance from a map of namespaces and genesis transactions
    pub fn new(networks: HashMap<String, PoolTxnAndConfig>) -> VdrResult<RunnerVdr> {
        let mut pools = HashMap::new();

        for (namespace, cfg) in networks {
            let txn = cfg.txn;
            let config = cfg.config;

            let pool_builder = if config.is_some() {
                PoolBuilder::new(config.unwrap(), None, None).transactions(txn.to_owned())?
            } else {
                PoolBuilder::default().transactions(txn.to_owned())?
            };
            let pool = pool_builder.into_runner()?;
            pools.insert(namespace.to_string(), pool);
        }
        Ok(RunnerVdr { pools })
    }

    /// Add a new ledger to the VDR
    pub fn add_ledger(&mut self, namespace: &str, pool: PoolRunner) {
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
    pub fn get_pool(&self, namespace: &str) -> Option<&PoolRunner> {
        self.borrow().pools.get(namespace)
    }

    pub fn set_pool(&mut self, namespace: String, pool: PoolRunner) {
        self.borrow_mut().pools.insert(namespace, pool);
    }
    /// Send a prepared request to a specific network
    // pub async fn send_request(
    //     &self,
    //     namespace: &str,
    //     req: PreparedRequest,
    // ) -> VdrResult<(RequestResult<String>, Option<TimingResult>)> {
    //     let pool = self
    //         .pools
    //         .get(namespace)
    //         .ok_or(err_msg(VdrErrorKind::Resolver, "Unkown namespace"))?;
    //     perform_ledger_request(pool, &req).await
    // }

    /// Dereference a DID Url and return a serialized `DereferencingResult`
    pub fn dereference(
        &self,
        did_url: &str,
        callback: Callback<VdrResult<String>>,
    ) -> VdrResult<()> {
        debug!("VDR: Dereference DID Url {}", did_url);
        let did_url = DidUrl::from_str(did_url)?;
        self._resolve(
            did_url,
            Box::new(|result| {
                match result {
                    Ok((data, metadata)) => {
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

                        callback(Ok(serde_json::to_string_pretty(&result).unwrap()));
                    }
                    // TODO: How to handle errors?
                    Err(_err) => {}
                }
            }),
        )?;
        Ok(())
    }

    /// Resolve a DID and return a serialized `ResolutionResult`
    pub fn resolve(&self, did: &str, callback: Callback<VdrResult<String>>) -> VdrResult<()> {
        debug!("VDR Resolve DID {}", did);
        let did = DidUrl::from_str(did)?;
        self._resolve(
            did,
            Box::new(|result| {
                match result {
                    Ok((data, metadata)) => {
                        match data {
                            // TODO: Doc needs to be mutable, when we uncomment
                            Result::DidDocument(doc) => {
                                // Try to find legacy endpoint using a GET_ATTRIB txn if diddoc_content is none
                                if doc.diddoc_content.is_none() {
                                    //     let pool = self
                                    //         .pools
                                    //         .get(&did.namespace)
                                    //         .ok_or(err_msg(VdrErrorKind::Resolver, "Unkown namespace"))
                                    //         .unwrap();
                                    //     fetch_legacy_endpoint_with_runner(
                                    //         pool,
                                    //         &did.id,
                                    //         Box::new(|result| {
                                    //             doc.endpoint = result.ok();
                                    //             let diddoc = Some(doc.to_value().unwrap());
                                    //             let md = if let Metadata::DidDocumentMetadata(md) =
                                    //                 metadata
                                    //             {
                                    //                 Some(md)
                                    //             } else {
                                    //                 None
                                    //             };

                                    //             let result = ResolutionResult {
                                    //                 did_resolution_metadata: None,
                                    //                 did_document: diddoc,
                                    //                 did_document_metadata: md,
                                    //             };

                                    //             callback(Ok(
                                    //                 serde_json::to_string_pretty(&result).unwrap()
                                    //             ))
                                    //         }),
                                    //     );
                                } else {
                                    // let diddoc = Some(doc.to_value().unwrap());
                                    // let md = if let Metadata::DidDocumentMetadata(md) = metadata {
                                    //     Some(md)
                                    // } else {
                                    //     None
                                    // };

                                    // let result = ResolutionResult {
                                    //     did_resolution_metadata: None,
                                    //     did_document: diddoc,
                                    //     did_document_metadata: md,
                                    // };
                                    // if !doc.diddoc_content.is_none() {
                                    //     callback(Ok(serde_json::to_string_pretty(&result).unwrap()))
                                    // }
                                }

                                // For now, until if/else above is used

                                let diddoc = Some(doc.to_value().unwrap());
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

                                callback(Ok(serde_json::to_string_pretty(&result).unwrap()))
                            }
                            _ => {}
                        };
                    }
                    // TODO: How to handle errors?
                    Err(_err) => {}
                }
            }),
        )?;
        Ok(())
    }

    // Internal method to resolve and dereference
    fn _resolve(
        &self,
        did_url: DidUrl,
        callback: Callback<VdrResult<(Result, Metadata)>>,
    ) -> VdrResult<()> {
        let runner = self
            .pools
            .get(&did_url.namespace)
            .ok_or(err_msg(VdrErrorKind::Resolver, "Unkown namespace"))?;

        let builder = RequestBuilder::default();
        let request = build_request(&did_url, &builder)?;

        let txn_type = request.txn_type.to_owned();

        runner.send_request(
            request,
            Box::new(
                move |result: VdrResult<(RequestResult<String>, Option<TimingResult>)>| {
                    let ledger_data = match result {
                        Ok((reply, _)) => match reply {
                            RequestResult::Reply(reply_data) => Ok(reply_data),
                            RequestResult::Failed(err) => Err(err),
                        },
                        Err(err) => Err(err),
                    }
                    .unwrap();

                    let namespace = did_url.namespace.clone();

                    let result = handle_resolution_result(
                        namespace.as_str(),
                        &ledger_data,
                        txn_type.as_str(),
                    )
                    .unwrap();
                    callback(Ok(result))
                },
            ),
        )
    }
}
