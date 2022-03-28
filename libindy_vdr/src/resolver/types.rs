use serde_json::Value as SJsonValue;

use super::did_document::DidDocument;

pub type Callback<R> = Box<dyn FnOnce(R) + Send>;

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
