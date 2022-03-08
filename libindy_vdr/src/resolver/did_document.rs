use crate::common::error::prelude::*;
use crate::ledger::responses::Endpoint;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value as SJsonValue};

pub const LEGACY_INDY_SERVICE: &str = "endpoint";
pub const DID_CORE_CONTEXT: &str = "https://www.w3.org/ns/did/v1";

#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Ed25519VerificationKey2018 {
    pub id: String,
    pub type_: String,
    pub controller: String,
    pub public_key_base58: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DidCommService {
    pub id: String,
    pub type_: String,
    pub recipient_keys: Vec<String>,
    pub routing_keys: Vec<String>,
    pub priority: u8,
}

impl DidCommService {
    pub fn new(id: String, recipient_keys: Vec<String>, routing_keys: Vec<String>) -> Self {
        Self {
            id,
            type_: "did-communication".to_string(),
            recipient_keys,
            routing_keys,
            priority: 0,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenericService {
    pub id: String,
    pub type_: String,
    pub service_endpoint: String,
}

#[derive(Serialize, PartialEq, Debug)]
pub enum Service {
    GenericService(GenericService),
    DidCommService(DidCommService),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DidDocument {
    namespace: String,
    id: String,
    verkey: String,
    endpoint: Option<Endpoint>,
    diddoc_content: Option<SJsonValue>,
}

impl DidDocument {
    pub fn new(
        namespace: &str,
        id: &str,
        verkey: &str,
        endpoint: Option<Endpoint>,
        diddoc_content: Option<SJsonValue>,
    ) -> Self {
        DidDocument {
            namespace: namespace.to_string(),
            id: id.to_string(),
            verkey: expand_verkey(id, verkey),
            endpoint,
            diddoc_content,
        }
    }

    pub fn to_value(&self) -> VdrResult<SJsonValue> {
        let mut doc = json!({
             "id": format!("did:indy:{}:{}", self.namespace, self.id),
            "verificationMethod": [Ed25519VerificationKey2018 {
                id: format!("did:indy:{}:{}#verkey", self.namespace, self.id),
                type_: format!("Ed25519VerificationKey2018"),
                controller: format!("did:indy:{}:{}", self.namespace, self.id),
                public_key_base58: format!("{}", self.verkey),
            }],
            "authentication": [format!("did:indy:{}:{}#verkey", self.namespace, self.id)],
        });

        if self.diddoc_content.is_some() {
            let is_valid = validate_diddoc_content(&(self.diddoc_content.as_ref().unwrap()));

            if is_valid {
                merge_diddoc(&mut doc, self.diddoc_content.as_ref().unwrap());
            } else {
                return Err(err_msg(VdrErrorKind::Resolver, "Invalid DIDdoc Content"));
            }

            // Handling of legacy services
        } else if self.endpoint.is_some() {
            let mut services = Vec::new();
            let endpoints = self.endpoint.clone();
            for (service, service_endpoint) in endpoints.unwrap().endpoint.into_iter() {
                let s = match service.as_str() {
                    LEGACY_INDY_SERVICE => json!(DidCommService::new(
                        format!("did:indy:{}:{}#did-communication", self.namespace, self.id),
                        vec![format!("did:indy:{}:{}#verkey", self.namespace, self.id)],
                        vec![],
                    )),
                    type_ => json!(GenericService {
                        id: format!("did:indy:{}:{}#{}", self.namespace, self.id, type_),
                        type_: type_.to_string(),
                        service_endpoint,
                    }),
                };
                services.push(s);
            }

            if let SJsonValue::Object(ref mut map) = doc {
                map.insert("service".to_string(), SJsonValue::Array(services));
            }
        }

        Ok(doc)
    }

    pub fn to_string(&self) -> VdrResult<String> {
        let doc = self.to_value()?;
        Ok(serde_json::to_string_pretty(&doc).unwrap())
    }
}

fn validate_context(context: &str) -> bool {
    context == DID_CORE_CONTEXT
}

fn validate_diddoc_content(diddoc_content: &SJsonValue) -> bool {
    if diddoc_content.get("id").is_some() {
        false
    } else if diddoc_content.get("@context").is_some() {
        let context = diddoc_content.get("@context").unwrap();

        if context.is_string() {
            validate_context(context.as_str().unwrap())
        } else if context.is_array() {
            let mut buf = false;
            for c in context.as_array().unwrap() {
                if buf {
                    return buf;
                }
                buf = validate_context(c.as_str().unwrap());
            }
            buf
        } else {
            false
        }
    } else {
        true
    }
}

fn merge_diddoc(base: &mut SJsonValue, content: &SJsonValue) {
    match (base, content) {
        (SJsonValue::Object(base), SJsonValue::Object(content)) => {
            for (k, v) in content {
                if k == "authentication" || k == "verificationMethod" {
                    let mut _tmp = base[k].as_array().unwrap().to_owned();
                    _tmp.append(&mut v.as_array().unwrap_or(&vec![v.to_owned()]).to_owned());
                    base[k] = SJsonValue::from(_tmp);
                } else {
                    merge_diddoc(base.entry(k).or_insert(SJsonValue::Null), v);
                }
            }
        }
        (a, b) => *a = b.clone(),
    }
}

fn expand_verkey(id: &str, verkey: &str) -> String {
    if verkey.starts_with('~') {
        format!("{}{}", id, &verkey[1..])
    } else {
        verkey.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn serialze_diddoc_without_diddoc_content() {
        let doc = DidDocument::new(
            "idunion",
            "QowxFtwciWceMFr7WbwnM",
            "67yDXtw6MK2D7V2kFSL7uMH6qTtrEbNtkdiTkbk9YJBk",
            None,
            None,
        );

        let serialized = json!({
            "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM",
            "verificationMethod": [{
                "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM#verkey",
                "type": "Ed25519VerificationKey2018",
                "controller": "did:indy:idunion:QowxFtwciWceMFr7WbwnM",
                "publicKeyBase58": "67yDXtw6MK2D7V2kFSL7uMH6qTtrEbNtkdiTkbk9YJBk",
            }],
            "authentication": ["did:indy:idunion:QowxFtwciWceMFr7WbwnM#verkey"],
        });

        // Need to compare serde value instead of string, since elements might be in
        // different order.

        let v_from_doc: SJsonValue =
            serde_json::from_str(doc.to_string().unwrap().as_str()).unwrap();
        let v_from_serialized: SJsonValue =
            serde_json::from_str(serde_json::to_string(&serialized).unwrap().as_str()).unwrap();

        assert_eq!(v_from_doc, v_from_serialized)
    }

    #[test]
    fn serialze_diddoc_with_diddoc_content() {
        let diddoc_content = json!({
        "@context" : [
            "https://www.w3.org/ns/did/v1",
            "https://identity.foundation/didcomm-messaging/service-endpoint/v1"
        ],
        "service": [
          {
            "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM#did-communication",
            "type": "did-communication",
            "serviceEndpoint": "https://example.com",
            "recipientKeys": [ "#verkey" ],
            "routingKeys": [ ],
            "priority": 0
          }
        ]
        });

        let doc = DidDocument::new(
            "idunion",
            "QowxFtwciWceMFr7WbwnM",
            "67yDXtw6MK2D7V2kFSL7uMH6qTtrEbNtkdiTkbk9YJBk",
            None,
            Some(diddoc_content),
        );

        let serialized = json!({
            "@context": [
              "https://www.w3.org/ns/did/v1",
               "https://identity.foundation/didcomm-messaging/service-endpoint/v1"
            ],
            "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM",
            "verificationMethod": [{
                "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM#verkey",
                "type": "Ed25519VerificationKey2018",
                "controller": "did:indy:idunion:QowxFtwciWceMFr7WbwnM",
                "publicKeyBase58": "67yDXtw6MK2D7V2kFSL7uMH6qTtrEbNtkdiTkbk9YJBk",
            }],
            "authentication": ["did:indy:idunion:QowxFtwciWceMFr7WbwnM#verkey"],
            "service": [{
                "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM#did-communication",
                "type": "did-communication",
                "serviceEndpoint": "https://example.com",
                "recipientKeys": [ "#verkey" ],
                "routingKeys": [],
                "priority": 0
            }]

        });

        let v_from_doc: SJsonValue =
            serde_json::from_str(doc.to_string().unwrap().as_str()).unwrap();
        let v_from_serialized: SJsonValue =
            serde_json::from_str(serde_json::to_string(&serialized).unwrap().as_str()).unwrap();

        assert_eq!(v_from_doc, v_from_serialized)
    }

    #[test]
    fn serialze_diddoc_with_diddoc_content_with_additional_auth() {
        let diddoc_content = json!({
        "@context" : [
            "https://www.w3.org/ns/did/v1",
            "https://identity.foundation/didcomm-messaging/service-endpoint/v1"
        ],
        "verificationMethod": [{
            "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM#keys-2",
            "type": "Ed25519VerificationKey2018",
            "controller": "did:indy:idunion:QowxFtwciWceMFr7WbwnM",
            "publicKeyBase58": "67yDXtw6MK2D7V2kFSL7uMH6qTtrEbNtkdiTkbk9YJBc",
        }],
        "authentication": ["did:indy:idunion:QowxFtwciWceMFr7WbwnM#keys-2"],
        "service": [{
            "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM#did-communication",
            "type": "did-communication",
            "serviceEndpoint": "https://example.com",
            "recipientKeys": [ "#verkey" ],
            "routingKeys": [],
            "priority": 0
        }]
        });

        let doc = DidDocument::new(
            "idunion",
            "QowxFtwciWceMFr7WbwnM",
            "67yDXtw6MK2D7V2kFSL7uMH6qTtrEbNtkdiTkbk9YJBk",
            None,
            Some(diddoc_content),
        );

        let serialized = json!({
            "@context": [
              "https://www.w3.org/ns/did/v1",
               "https://identity.foundation/didcomm-messaging/service-endpoint/v1"
            ],
            "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM",
            "verificationMethod": [{
                "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM#verkey",
                "type": "Ed25519VerificationKey2018",
                "controller": "did:indy:idunion:QowxFtwciWceMFr7WbwnM",
                "publicKeyBase58": "67yDXtw6MK2D7V2kFSL7uMH6qTtrEbNtkdiTkbk9YJBk",
            },{
                "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM#keys-2",
                "type": "Ed25519VerificationKey2018",
                "controller": "did:indy:idunion:QowxFtwciWceMFr7WbwnM",
                "publicKeyBase58": "67yDXtw6MK2D7V2kFSL7uMH6qTtrEbNtkdiTkbk9YJBc",
            }],
            "authentication": [
                "did:indy:idunion:QowxFtwciWceMFr7WbwnM#verkey",
                "did:indy:idunion:QowxFtwciWceMFr7WbwnM#keys-2"],
                "service": [{
                    "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM#did-communication",
                    "type": "did-communication",
                    "serviceEndpoint": "https://example.com",
                    "recipientKeys": [ "#verkey" ],
                    "routingKeys": [],
                    "priority": 0
                }]

        });

        let v_from_doc: SJsonValue =
            serde_json::from_str(doc.to_string().unwrap().as_str()).unwrap();
        let v_from_serialized: SJsonValue =
            serde_json::from_str(serde_json::to_string(&serialized).unwrap().as_str()).unwrap();

        assert_eq!(v_from_doc, v_from_serialized)
    }

    #[test]
    fn serialze_diddoc_with_diddoc_content_with_additional_auth_as_string() {
        let diddoc_content = json!({
        "@context" : [
            "https://www.w3.org/ns/did/v1",
            "https://identity.foundation/didcomm-messaging/service-endpoint/v1"
        ],
        "verificationMethod": [{
            "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM#keys-2",
            "type": "Ed25519VerificationKey2018",
            "controller": "did:indy:idunion:QowxFtwciWceMFr7WbwnM",
            "publicKeyBase58": "67yDXtw6MK2D7V2kFSL7uMH6qTtrEbNtkdiTkbk9YJBc",
        }],
        "authentication": "did:indy:idunion:QowxFtwciWceMFr7WbwnM#keys-2",
        "service": [{
            "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM#did-communication",
            "type": "did-communication",
            "serviceEndpoint": "https://example.com",
            "recipientKeys": [ "#verkey" ],
            "routingKeys": [],
            "priority": 0
        }]
        });

        let doc = DidDocument::new(
            "idunion",
            "QowxFtwciWceMFr7WbwnM",
            "67yDXtw6MK2D7V2kFSL7uMH6qTtrEbNtkdiTkbk9YJBk",
            None,
            Some(diddoc_content),
        );

        let serialized = json!({
            "@context": [
              "https://www.w3.org/ns/did/v1",
               "https://identity.foundation/didcomm-messaging/service-endpoint/v1"
            ],
            "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM",
            "verificationMethod": [{
                "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM#verkey",
                "type": "Ed25519VerificationKey2018",
                "controller": "did:indy:idunion:QowxFtwciWceMFr7WbwnM",
                "publicKeyBase58": "67yDXtw6MK2D7V2kFSL7uMH6qTtrEbNtkdiTkbk9YJBk",
            },{
                "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM#keys-2",
                "type": "Ed25519VerificationKey2018",
                "controller": "did:indy:idunion:QowxFtwciWceMFr7WbwnM",
                "publicKeyBase58": "67yDXtw6MK2D7V2kFSL7uMH6qTtrEbNtkdiTkbk9YJBc",
            }],
            "authentication": [
                "did:indy:idunion:QowxFtwciWceMFr7WbwnM#verkey",
                "did:indy:idunion:QowxFtwciWceMFr7WbwnM#keys-2"
                ],
                "service": [{
                    "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM#did-communication",
                    "type": "did-communication",
                    "serviceEndpoint": "https://example.com",
                    "recipientKeys": [ "#verkey" ],
                    "routingKeys": [],
                    "priority": 0
                }]

        });

        let v_from_doc: SJsonValue =
            serde_json::from_str(doc.to_string().unwrap().as_str()).unwrap();
        let v_from_serialized: SJsonValue =
            serde_json::from_str(serde_json::to_string(&serialized).unwrap().as_str()).unwrap();

        assert_eq!(v_from_doc, v_from_serialized)
    }

    #[test]
    fn serialze_diddoc_with_legacy_did_comm_endpoint() {
        let mut endpoint_map: HashMap<String, String> = HashMap::new();
        endpoint_map.insert(String::from("endpoint"), String::from("https://agent.com"));

        let doc = DidDocument::new(
            "idunion",
            "QowxFtwciWceMFr7WbwnM",
            "67yDXtw6MK2D7V2kFSL7uMH6qTtrEbNtkdiTkbk9YJBk",
            Some(Endpoint {
                endpoint: endpoint_map,
            }),
            None,
        );

        let serialized = json!({
            "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM",
            "verificationMethod": [{
                "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM#verkey",
                "type": "Ed25519VerificationKey2018",
                "controller": "did:indy:idunion:QowxFtwciWceMFr7WbwnM",
                "publicKeyBase58": "67yDXtw6MK2D7V2kFSL7uMH6qTtrEbNtkdiTkbk9YJBk",
            }],
            "authentication": ["did:indy:idunion:QowxFtwciWceMFr7WbwnM#verkey"],
            "service": [{
                "id": "did:indy:idunion:QowxFtwciWceMFr7WbwnM#did-communication",
                "type": "did-communication",
                "recipientKeys": ["did:indy:idunion:QowxFtwciWceMFr7WbwnM#verkey"],
                "routingKeys": [],
                "priority": 0
            }]

        });

        let v_from_doc: SJsonValue =
            serde_json::from_str(doc.to_string().unwrap().as_str()).unwrap();
        let v_from_serialized: SJsonValue =
            serde_json::from_str(serde_json::to_string(&serialized).unwrap().as_str()).unwrap();

        assert_eq!(v_from_doc, v_from_serialized)
    }

    #[test]
    fn serialze_diddoc_with_multiple_legacy_endpoints() {
        let mut endpoint_map: HashMap<String, String> = HashMap::new();
        endpoint_map.insert(String::from("endpoint"), String::from("https://agent.com"));
        endpoint_map.insert(
            String::from("profile"),
            String::from("https://agent.com/profile"),
        );

        let doc = DidDocument::new(
            "idunion",
            "QowxFtwciWceMFr7WbwnM",
            "67yDXtw6MK2D7V2kFSL7uMH6qTtrEbNtkdiTkbk9YJBk",
            Some(Endpoint {
                endpoint: endpoint_map,
            }),
            None,
        );

        let v_from_doc: SJsonValue =
            serde_json::from_str(doc.to_string().unwrap().as_str()).unwrap();

        assert_eq!(2, v_from_doc["service"].as_array().unwrap().len())
    }

    #[test]
    fn validate_diddoc_with_context_as_string() {
        let diddoc_content = json!({
            "@context" : "https://www.w3.org/ns/did/v1"
        });
        assert!(validate_diddoc_content(&diddoc_content))
    }

    #[test]
    fn validate_diddoc_without_context() {
        let diddoc_content = json!({
        "service": [
          {
            "id": "did:indy:sovrin:123456#did-communication",
            "type": "did-communication",
            "serviceEndpoint": "https://example.com",
            "recipientKeys": [ "#verkey" ],
            "routingKeys": [ ]
          }
        ]
        });
        assert!(validate_diddoc_content(&diddoc_content))
    }

    #[test]
    fn validate_diddoc_with_context_as_array() {
        let diddoc_content = json!({
            "@context" : [
                "https://www.w3.org/ns/did/v1",
                "https://identity.foundation/didcomm-messaging/service-endpoint/v1"
        ],
        });
        assert!(validate_diddoc_content(&diddoc_content))
    }

    #[test]
    fn validate_diddoc_with_empty_context_as_array() {
        let diddoc_content = json!({
            "@context" : [],
        });
        assert!(!validate_diddoc_content(&diddoc_content))
    }

    #[test]
    fn validate_diddoc_with_empty_context_as_string() {
        let diddoc_content = json!({
            "@context" : "",
        });
        assert!(!validate_diddoc_content(&diddoc_content))
    }

    #[test]
    fn validate_diddoc_with_id() {
        let diddoc_content = json!({
            "id" : "sg3535sd",
        });
        assert!(!validate_diddoc_content(&diddoc_content))
    }
}
