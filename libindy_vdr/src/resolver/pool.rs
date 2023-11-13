use super::did::DidUrl;
use crate::common::error::prelude::*;

use crate::ledger::RequestBuilder;
use crate::pool::{Pool, PoolRunner, RequestResult, TimingResult};

use super::types::*;
use super::utils::*;

/// DID (URL) Resolver interface for a pool compliant with did:indy method spec
/// The resolver interface is bound to a specific indy network and does not evaluate
/// the namespace part of the DID. You need to create a resolver instance for each
/// indy network you want to support.
/// The `PoolResolver` uses async/await.
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
        let did_url = DidUrl::parse(did_url)?;
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
        let did = DidUrl::parse(did)?;
        let (data, metadata) = self._resolve(&did).await?;

        let md = if let Metadata::DidDocumentMetadata(md) = metadata {
            Some(md)
        } else {
            None
        };

        let diddoc = match data {
            Result::DidDocument(mut doc) => {
                // Try to find legacy endpoint using a GET_ATTRIB txn if diddoc_content is none
                if doc.diddoc_content.is_none() {
                    let (seq_no, timestamp) = if let Some(md) = &md {
                        let j_seq_no = &md.node_response["result"]["seqNo"];
                        let j_timestamp = &md.node_response["result"]["timestamp"];

                        let mut seq_no = None;
                        let mut timestamp = None;

                        if !j_seq_no.is_null() {
                            seq_no = j_seq_no.as_u64().map(|v| v as i32);
                        }

                        if !j_timestamp.is_null() {
                            timestamp = j_timestamp.as_u64();
                            // Use timestamp if possible
                            if timestamp.is_some() {
                                seq_no = None;
                            }
                        }
                        (seq_no, timestamp)
                    } else {
                        (None, None)
                    };
                    doc.endpoint = fetch_legacy_endpoint(&self.pool, &did.id, seq_no, timestamp)
                        .await
                        .ok();
                }
                Some(doc.to_value()?)
            }
            _ => None,
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
        let request = build_request(did_url, &builder)?;
        debug!(
            "PoolResolver: Prepared Request for DID {}: {:#?}",
            did_url.id, request
        );
        let ledger_data = handle_request(&self.pool, &request).await?;
        let namespace = did_url.namespace.clone();
        let result = handle_internal_resolution_result(namespace.as_str(), &ledger_data)?;

        Ok(result)
    }
}

/// DID (URL) Resolver interface using callbacks for a PoolRunner compliant with did:indy method spec
/// The PoolRunnerResolver is used for the FFI.
/// Note that the PoolRunnerResolver does not fetch an ATTRIB txn for legacy endpoint resolution.
/// If you need to use the PoolRunnerResolver, please have a look at the Python wrapper to see how
/// legacy endpoints can be resolved.
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
        did_url: String,
        callback: Callback<SendReqResponse>,
    ) -> VdrResult<()> {
        let did_url = DidUrl::parse(did_url.as_str())?;
        self._resolve(&did_url, callback)?;
        Ok(())
    }

    /// Resolve a DID and return a serialized `ResolutionResult`
    pub fn resolve(&self, did: String, callback: Callback<SendReqResponse>) -> VdrResult<()> {
        let did = DidUrl::parse(did.as_str())?;
        self._resolve(&did, callback)?;
        Ok(())
    }

    fn _resolve(&self, did_url: &DidUrl, callback: Callback<SendReqResponse>) -> VdrResult<()> {
        let builder = RequestBuilder::default();
        let request = build_request(did_url, &builder)?;
        self.runner.send_request(request, callback)
    }
}

type SendReqResponse = VdrResult<(RequestResult<String>, Option<TimingResult>)>;

pub fn handle_resolution_result(result: SendReqResponse, did_url: String) -> VdrResult<String> {
    let did = DidUrl::parse(did_url.as_str())?;
    let (req_result, _timing_result) = result?;

    let ledger_data = match req_result {
        RequestResult::Reply(reply_data) => Ok(reply_data),
        RequestResult::Failed(err) => Err(err),
    }?;

    let namespace = did.namespace;

    let (data, metadata) = handle_internal_resolution_result(namespace.as_str(), &ledger_data)?;

    let content = match data {
        Result::Content(c) => Some(c),
        Result::DidDocument(doc) => doc.to_value().ok(),
    };

    match metadata {
        Metadata::ContentMetadata(md) => {
            let result = DereferencingResult {
                dereferencing_metadata: None,
                content_stream: content,
                content_metadata: Some(md),
            };

            serde_json::to_string_pretty(&result)
                .map_err(|err| err_msg(VdrErrorKind::Unexpected, err))
        }
        Metadata::DidDocumentMetadata(md) => {
            let result = ResolutionResult {
                did_resolution_metadata: None,
                did_document: content,
                did_document_metadata: Some(md),
            };

            serde_json::to_string_pretty(&result)
                .map_err(|err| err_msg(VdrErrorKind::Unexpected, err))
        }
    }
}
