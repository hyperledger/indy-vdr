use super::did::DidUrl;
use crate::common::error::prelude::*;

use crate::ledger::RequestBuilder;
use crate::pool::{Pool, PoolRunner, RequestResult, TimingResult};

use super::types::*;
use super::utils::*;

/// DID (URL) Resolver interface for a pool compliant with did:indy method spec
pub struct PoolResolver<T: Pool> {
    pool: T,
}

impl<T: Pool> PoolResolver<T> {
    pub fn new(pool: T) -> PoolResolver<T> {
        PoolResolver { pool }
    }

    /// Dereference a DID Url and return a serialized `DereferencingResult`
    pub async fn dereference(&self, did_url: &str) -> VdrResult<String> {
        debug!("PoolResolver: Dereference DID Url {}", did_url);
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
        debug!("PoolResolver: Resolve DID {}", did);
        let did = DidUrl::from_str(did)?;
        let (data, metadata) = self._resolve(&did).await?;

        let diddoc = match data {
            Result::DidDocument(mut doc) => {
                // Try to find legacy endpoint using a GET_ATTRIB txn if diddoc_content is none
                if doc.diddoc_content.is_none() {
                    doc.endpoint = fetch_legacy_endpoint(&self.pool, &did.id).await.ok();
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
        let builder = self.pool.get_request_builder();
        let request = build_request(&did_url, &builder)?;

        let ledger_data = handle_request(&self.pool, &request).await?;
        let txn_type = request.txn_type.as_str();
        let namespace = did_url.namespace.clone();
        let result = handle_resolution_result(namespace.as_str(), &ledger_data, txn_type)?;

        Ok(result)
    }
}

/// DID (URL) Resolver interface using callbacks for a PoolRunner compliant with did:indy method spec
pub struct PoolRunnerResolver<'a> {
    runner: &'a PoolRunner,
}

impl<'a> PoolRunnerResolver<'a> {
    pub fn new(runner: &'a PoolRunner) -> PoolRunnerResolver {
        PoolRunnerResolver { runner }
    }

    /// Dereference a DID Url and return a serialized `DereferencingResult`
    pub fn dereference(
        &self,
        did_url: &str,
        callback: Callback<VdrResult<String>>,
    ) -> VdrResult<()> {
        let did_url = DidUrl::from_str(did_url)?;
        self._resolve(
            &did_url,
            Box::new(move |result| {
                let (data, metadata) = result.unwrap();
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

                callback(Ok(serde_json::to_string_pretty(&result).unwrap()))
            }),
        )
    }

    /// Resolve a DID and return a serialized `ResolutionResult`
    pub fn resolve(&self, did: &str, callback: Callback<VdrResult<String>>) -> VdrResult<()> {
        let did = DidUrl::from_str(did)?;
        self._resolve(
            &did,
            Box::new(move |result| {
                match result {
                    Ok((data, metadata)) => {
                        match data {
                            // TODO: Doc needs to be mutable, when we uncomment
                            Result::DidDocument(doc) => {
                                // Try to find legacy endpoint using a GET_ATTRIB txn if diddoc_content is none
                                if doc.diddoc_content.is_none() {
                                    // let did_copy = did.id.clone();
                                    // fetch_legacy_endpoint_with_runner(
                                    //     self.runner,
                                    //     &did_copy,
                                    //     Box::new(|result| {
                                    //         doc.endpoint = result.ok();
                                    //         let diddoc = Some(doc.to_value().unwrap());
                                    //         let md = if let Metadata::DidDocumentMetadata(md) =
                                    //             metadata
                                    //         {
                                    //             Some(md)
                                    //         } else {
                                    //             None
                                    //         };

                                    //         let result = ResolutionResult {
                                    //             did_resolution_metadata: None,
                                    //             did_document: diddoc,
                                    //             did_document_metadata: md,
                                    //         };

                                    //         callback(Ok(
                                    //             serde_json::to_string_pretty(&result).unwrap()
                                    //         ))
                                    //     }),
                                    // );
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

                                    //     callback(Ok(serde_json::to_string_pretty(&result).unwrap()))
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

    fn _resolve(
        &self,
        did_url: &DidUrl,
        callback: Callback<VdrResult<(Result, Metadata)>>,
    ) -> VdrResult<()> {
        let namespace = did_url.namespace.clone();

        let builder = RequestBuilder::default();
        let request = build_request(&did_url, &builder)?;
        let txn_type = request.txn_type.clone();

        self.runner.send_request(
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
