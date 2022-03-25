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
        let namespace = did_url.namespace.clone();
        let result = handle_internal_resolution_result(namespace.as_str(), &ledger_data)?;

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
        did_url: String,
        callback: Callback<SendReqResponse>,
    ) -> VdrResult<()> {
        let did_url = DidUrl::from_str(did_url.as_str())?;
        self._resolve(&did_url, callback)?;
        Ok(())
    }

    /// Resolve a DID and return a serialized `ResolutionResult`
    pub fn resolve(&self, did: String, callback: Callback<SendReqResponse>) -> VdrResult<()> {
        let did = DidUrl::from_str(did.as_str())?;
        self._resolve(&did, callback)?;
        Ok(())
    }

    fn _resolve(&self, did_url: &DidUrl, callback: Callback<SendReqResponse>) -> VdrResult<()> {
        // let namespace = did_url.namespace.clone();

        let builder = RequestBuilder::default();
        let request = build_request(&did_url, &builder)?;
        // let txn_type = request.txn_type.clone();

        self.runner.send_request(request, callback)
    }
}

type SendReqResponse = VdrResult<(RequestResult<String>, Option<TimingResult>)>;

pub fn handle_resolution_result(result: SendReqResponse, did_url: String) -> VdrResult<String> {
    let did = DidUrl::from_str(did_url.as_str())?;
    let (req_result, _timing_result) = result?;

    let ledger_data = match req_result {
        RequestResult::Reply(reply_data) => Ok(reply_data),
        RequestResult::Failed(err) => Err(err),
    }?;

    println!("{:#?}", ledger_data);

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
