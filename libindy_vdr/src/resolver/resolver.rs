use serde_json::Value as SJsonValue;

use super::did::DidUrl;
use super::did_document::DidDocument;
use super::utils::*;

use crate::common::error::prelude::*;

use crate::ledger::RequestBuilder;
use crate::pool::{Pool, PoolRunner, RequestResult, TimingResult};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Result {
    DidDocument(DidDocument),
    Content(SJsonValue),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Metadata {
    DidDocumentMetadata(DidDocumentMetadata),
    ContentMetadata(ContentMetadata),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentMetadata {
    pub node_response: SJsonValue,
    pub object_type: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DidDocumentMetadata {
    pub node_response: SJsonValue,
    pub object_type: String,
    pub self_certification_version: Option<i32>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionResult {
    pub did_resolution_metadata: Option<String>,
    pub did_document: Option<SJsonValue>,
    pub did_document_metadata: Option<DidDocumentMetadata>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DereferencingResult {
    pub dereferencing_metadata: Option<String>,
    pub content_stream: Option<SJsonValue>,
    pub content_metadata: Option<ContentMetadata>,
}

pub trait Resolver {
    fn resolve(&self, did_url: &str) -> VdrResult<String>;

    fn dereference(&self, did_url: &str) -> VdrResult<String>;
}

pub trait CallbackResolver {}

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
        let txn_type = &request.txn_type.as_str();
        let result = handle_resolution_result(&did_url, &ledger_data, txn_type)?;

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
        self._resolve(
            did_url,
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
                                    if !doc.diddoc_content.is_none() {
                                        callback(Ok(serde_json::to_string_pretty(&result).unwrap()))
                                    }
                                }
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

    // TODO: Refactor
    fn _resolve(
        &self,
        did: &str,
        callback: Callback<VdrResult<(Result, Metadata)>>,
    ) -> VdrResult<()> {
        let did_url = DidUrl::from_str(did)?;

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

                    let result =
                        handle_resolution_result(&did_url, &ledger_data, txn_type.as_str())
                            .unwrap();
                    callback(Ok(result))
                },
            ),
        )
    }
}
