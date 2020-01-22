use std::cmp::Eq;
use std::collections::HashMap;

use serde_json;
use ursa::bls::VerKey as BlsVerKey;

use crate::common::error::prelude::*;
use crate::common::verkey::VerKey;
use crate::config::constants::DEFAULT_PROTOCOL_VERSION;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProtocolVersion {
    Node1_3 = 1,
    Node1_4 = 2,
}

impl ProtocolVersion {
    pub fn display_version(&self) -> String {
        match self {
            Self::Node1_3 => "1.3".to_owned(),
            Self::Node1_4 => "1.4".to_owned(),
        }
    }

    pub fn to_id(&self) -> usize {
        *self as usize
    }
}

impl PartialEq<usize> for ProtocolVersion {
    fn eq(&self, other: &usize) -> bool {
        (*self as usize) == *other
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        DEFAULT_PROTOCOL_VERSION
    }
}

impl std::fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.display_version())
    }
}

pub type NodeKeys = HashMap<String, Option<BlsVerKey>>;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct NodeData {
    pub alias: String,
    pub client_ip: Option<String>,
    #[serde(deserialize_with = "string_or_number")]
    #[serde(default)]
    pub client_port: Option<u64>,
    pub node_ip: Option<String>,
    #[serde(deserialize_with = "string_or_number")]
    #[serde(default)]
    pub node_port: Option<u64>,
    pub services: Option<Vec<String>>,
    pub blskey: Option<String>,
    pub blskey_pop: Option<String>,
}

fn string_or_number<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let deser_res: Result<serde_json::Value, _> = serde::Deserialize::deserialize(deserializer);

    match deser_res {
        Ok(serde_json::Value::String(s)) => match s.parse::<u64>() {
            Ok(num) => Ok(Some(num)),
            Err(err) => Err(serde::de::Error::custom(format!(
                "Invalid Node transaction: {:?}",
                err
            ))),
        },
        Ok(serde_json::Value::Number(n)) => match n.as_u64() {
            Some(num) => Ok(Some(num)),
            None => Err(serde::de::Error::custom(
                "Invalid Node transaction".to_string(),
            )),
        },
        Ok(serde_json::Value::Null) => Ok(None),
        _ => Err(serde::de::Error::custom(
            "Invalid Node transaction".to_string(),
        )),
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum NodeTransaction {
    NodeTransactionV0(NodeTransactionV0),
    NodeTransactionV1(NodeTransactionV1),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct NodeTransactionV0 {
    pub data: NodeData,
    pub dest: String,
    pub identifier: String,
    #[serde(rename = "txnId")]
    pub txn_id: Option<String>,
    pub verkey: Option<String>,
    #[serde(rename = "type")]
    pub txn_type: String,
}

impl NodeTransactionV0 {
    pub const VERSION: &'static str = "1.3";
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NodeTransactionV1 {
    pub txn: Txn,
    pub txn_metadata: Metadata,
    pub req_signature: ReqSignature,
    pub ver: String,
}

impl NodeTransactionV1 {
    pub const VERSION: &'static str = "1.4";
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Txn {
    #[serde(rename = "type")]
    pub txn_type: String,
    #[serde(rename = "protocolVersion")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<i32>,
    pub data: TxnData,
    pub metadata: TxnMetadata,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creation_time: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq_no: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txn_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReqSignature {
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<ReqSignatureValue>>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ReqSignatureValue {
    pub from: Option<String>,
    pub value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct TxnData {
    pub data: NodeData,
    pub dest: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verkey: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TxnMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub req_id: Option<u64>,
    pub from: String,
}

impl From<NodeTransactionV0> for NodeTransactionV1 {
    fn from(node_txn: NodeTransactionV0) -> Self {
        {
            let txn = Txn {
                txn_type: node_txn.txn_type,
                protocol_version: None,
                data: TxnData {
                    data: node_txn.data,
                    dest: node_txn.dest,
                    verkey: node_txn.verkey,
                },
                metadata: TxnMetadata {
                    req_id: None,
                    from: node_txn.identifier,
                },
            };
            NodeTransactionV1 {
                txn,
                txn_metadata: Metadata {
                    seq_no: None,
                    txn_id: node_txn.txn_id,
                    creation_time: None,
                },
                req_signature: ReqSignature {
                    type_: None,
                    values: None,
                },
                ver: "1".to_string(),
            }
        }
    }
}

impl NodeTransactionV1 {
    pub fn update(&mut self, other: &mut NodeTransactionV1) -> LedgerResult<()> {
        assert_eq!(self.txn.data.dest, other.txn.data.dest);
        assert_eq!(self.txn.data.data.alias, other.txn.data.data.alias);

        if let Some(ref mut client_ip) = other.txn.data.data.client_ip {
            self.txn.data.data.client_ip = Some(client_ip.to_owned());
        }

        if let Some(ref mut client_port) = other.txn.data.data.client_port {
            self.txn.data.data.client_port = Some(client_port.to_owned());
        }

        if let Some(ref mut node_ip) = other.txn.data.data.node_ip {
            self.txn.data.data.node_ip = Some(node_ip.to_owned());
        }

        if let Some(ref mut node_port) = other.txn.data.data.node_port {
            self.txn.data.data.node_port = Some(node_port.to_owned());
        }

        if let Some(ref mut blskey) = other.txn.data.data.blskey {
            self.txn.data.data.blskey = Some(blskey.to_owned());
        }

        if let Some(ref mut blskey_pop) = other.txn.data.data.blskey_pop {
            self.txn.data.data.blskey_pop = Some(blskey_pop.to_owned());
        }

        if let Some(ref mut services) = other.txn.data.data.services {
            self.txn.data.data.services = Some(services.to_owned());
        }

        if other.txn.data.verkey.is_some() {
            let verkey = VerKey::from_str_qualified(
                other.txn.data.verkey.as_ref().unwrap().as_str(),
                Some(self.txn.data.dest.as_str()),
                None,
                None,
            )?;
            self.txn.data.verkey = Some(verkey.long_form());
        }

        Ok(())
    }
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LedgerStatus {
    pub txnSeqNo: usize,
    pub merkleRoot: String,
    pub ledgerId: u8,
    pub ppSeqNo: Option<u32>,
    pub viewNo: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocolVersion: Option<usize>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsistencyProof {
    //TODO almost all fields Option<> or find better approach
    pub seqNoEnd: usize,
    pub seqNoStart: usize,
    pub ledgerId: usize,
    pub hashes: Vec<String>,
    pub oldMerkleRoot: String,
    pub newMerkleRoot: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct CatchupReq {
    pub ledgerId: usize,
    pub seqNoStart: usize,
    pub seqNoEnd: usize,
    pub catchupTill: usize,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CatchupRep {
    pub ledgerId: usize,
    pub consProof: Vec<String>,
    pub txns: HashMap<String, serde_json::Value>,
}

impl CatchupRep {
    pub fn load_txns(&self) -> LedgerResult<Vec<Vec<u8>>> {
        let mut keys = self
            .txns
            .keys()
            .map(|k| {
                k.parse::<usize>().to_result(
                    LedgerErrorKind::InvalidStructure,
                    "Invalid key in catchup reply",
                )
            })
            .collect::<LedgerResult<Vec<usize>>>()?;
        keys.sort();
        Ok(keys
            .iter()
            .flat_map(|k| {
                let txn = self.txns.get(&k.to_string()).unwrap();
                rmp_serde::to_vec_named(txn).to_result(
                    LedgerErrorKind::InvalidStructure,
                    "Invalid transaction -- can not transform to bytes",
                )
            })
            .collect())
    }

    pub fn min_tx(&self) -> LedgerResult<usize> {
        let mut min = None;

        for (k, _) in self.txns.iter() {
            let val = k.parse::<usize>().to_result(
                LedgerErrorKind::InvalidStructure,
                "Invalid key in catchup reply",
            )?;

            match min {
                None => min = Some(val),
                Some(m) => {
                    if val < m {
                        min = Some(val)
                    }
                }
            }
        }

        min.ok_or_else(|| err_msg(LedgerErrorKind::InvalidStructure, "Empty map"))
    }
}

#[derive(Serialize, Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Reply {
    ReplyV0(ReplyV0),
    ReplyV1(ReplyV1),
}

impl Reply {
    pub fn req_id(&self) -> u64 {
        match *self {
            Reply::ReplyV0(ref reply) => reply.result.req_id,
            Reply::ReplyV1(ref reply) => reply.result.txn.metadata.req_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplyV0 {
    pub result: ResponseMetadata,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplyV1 {
    pub result: ReplyResultV1,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplyResultV1 {
    pub txn: ReplyTxnV1,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplyTxnV1 {
    pub metadata: ResponseMetadata,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Response {
    ResponseV0(ResponseV0),
    ResponseV1(ResponseV1),
}

impl Response {
    pub fn req_id(&self) -> u64 {
        match *self {
            Response::ResponseV0(ref res) => res.req_id,
            Response::ResponseV1(ref res) => res.metadata.req_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResponseV0 {
    pub req_id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseV1 {
    pub metadata: ResponseMetadata,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMetadata {
    pub req_id: u64,
}

#[derive(Serialize, Debug, Deserialize)]
#[serde(untagged)]
pub enum PoolLedgerTxn {
    PoolLedgerTxnV0(PoolLedgerTxnV0),
    PoolLedgerTxnV1(PoolLedgerTxnV1),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PoolLedgerTxnV0 {
    pub txn: Response,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PoolLedgerTxnV1 {
    pub txn: PoolLedgerTxnDataV1,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PoolLedgerTxnDataV1 {
    pub txn: Response,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SimpleRequest {
    pub req_id: u64,
}

#[serde(tag = "op")]
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    #[serde(rename = "CONSISTENCY_PROOF")]
    ConsistencyProof(ConsistencyProof),
    #[serde(rename = "LEDGER_STATUS")]
    LedgerStatus(LedgerStatus),
    #[serde(rename = "CATCHUP_REQ")]
    CatchupReq(CatchupReq),
    #[serde(rename = "CATCHUP_REP")]
    CatchupRep(CatchupRep),
    #[serde(rename = "REQACK")]
    ReqACK(Response),
    #[serde(rename = "REQNACK")]
    ReqNACK(Response),
    #[serde(rename = "REPLY")]
    Reply(Reply),
    #[serde(rename = "REJECT")]
    Reject(Response),
    #[serde(rename = "POOL_LEDGER_TXNS")]
    PoolLedgerTxns(PoolLedgerTxn),
    Ping,
    Pong,
}

impl Message {
    pub fn from_raw_str(str: &str) -> LedgerResult<Message> {
        match str {
            "po" => Ok(Message::Pong),
            "pi" => Ok(Message::Ping),
            _ => serde_json::from_str::<Message>(str)
                .to_result(LedgerErrorKind::InvalidStructure, "Malformed message json"),
        }
    }

    pub fn request_id(&self) -> Option<String> {
        match self {
            Message::Reply(ref rep) => Some(rep.req_id().to_string()),
            Message::ReqACK(ref rep) | Message::ReqNACK(ref rep) | Message::Reject(ref rep) => {
                Some(rep.req_id().to_string())
            }
            _ => None,
        }
    }

    pub fn serialize(&self) -> LedgerResult<serde_json::Value> {
        serde_json::to_value(&self).map_err(|err| {
            err_msg(
                LedgerErrorKind::InvalidStructure,
                format!("Cannot serialize Request: {:?}", err),
            )
        })
    }
}
