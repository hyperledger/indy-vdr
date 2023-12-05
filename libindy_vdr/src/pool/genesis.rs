use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::iter::IntoIterator;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rand::random;
use serde_json::{self, Deserializer, Value as SJsonValue};

use super::types::{
    NodeTransaction, NodeTransactionV0, NodeTransactionV1, ProtocolVersion, VerifierInfo,
    VerifierKey, Verifiers,
};
use crate::common::error::prelude::*;
use crate::common::merkle_tree::MerkleTree;
use crate::utils::{
    base58,
    keys::{EncodedVerKey, KeyEncoding, KeyType},
};

pub type NodeTransactionMap = HashMap<String, NodeTransactionV1>;

#[cfg(windows)]
const LINE_ENDING: &str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &str = "\n";

/// A collection of pool genesis transactions.
#[derive(Clone, PartialEq, Eq)]
pub struct PoolTransactions {
    inner: Vec<Vec<u8>>, // stored in msgpack format
}

impl PoolTransactions {
    /// Create a new blank set of transactions.
    fn new(inner: Vec<Vec<u8>>) -> Self {
        Self { inner }
    }

    /// Load JSON pool transactions from a string.
    pub fn from_json(txns: &str) -> VdrResult<Self> {
        let stream = Deserializer::from_str(txns).into_iter::<SJsonValue>();
        let txns = _json_iter_to_msgpack(stream)?;
        if txns.is_empty() {
            Err(input_err("No genesis transactions found"))
        } else {
            Ok(Self::new(txns))
        }
    }

    /// Load JSON pool transactions from a file.
    pub fn from_json_file<P>(file_name: P) -> VdrResult<Self>
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        let f = File::open(&file_name).map_err(|err| {
            err_msg(
                VdrErrorKind::FileSystem,
                format!("Can't open genesis transactions file: {:?}", file_name),
            )
            .with_source(err)
        })?;
        let reader = BufReader::new(&f);
        let stream = Deserializer::from_reader(reader).into_iter::<SJsonValue>();
        let txns = _json_iter_to_msgpack(stream)?;
        if txns.is_empty() {
            Err(input_err("No genesis transactions found"))
        } else {
            Ok(Self::new(txns))
        }
    }

    /// Load pool transactions from a sequence of JSON strings.
    pub fn from_json_transactions<T>(txns: T) -> VdrResult<Self>
    where
        T: IntoIterator,
        T::Item: AsRef<str>,
    {
        let mut pt = Self { inner: vec![] };
        pt.extend_from_json(txns)?;
        Ok(pt)
    }

    /// Load pool transactions from a sequence of msgpack-encoded byte strings.
    pub fn from_transactions<T>(txns: T) -> Self
    where
        T: IntoIterator,
        T::Item: AsRef<[u8]>,
    {
        Self::new(txns.into_iter().map(|t| t.as_ref().to_vec()).collect())
    }

    /// Extend the pool transactions with a set of msgpack-encoded byte strings.
    pub fn extend<T>(&mut self, txns: T)
    where
        T: IntoIterator<Item = Vec<u8>>,
    {
        self.inner.extend(txns)
    }

    /// Extend the pool transactions with a set of JSON strings.
    pub fn extend_from_json<T>(&mut self, txns: T) -> VdrResult<()>
    where
        T: IntoIterator,
        T::Item: AsRef<str>,
    {
        for txn in txns {
            let txn = serde_json::from_str::<SJsonValue>(txn.as_ref())
                .with_input_err("Error deserializing transaction as JSON")?;
            self.inner.push(_json_to_msgpack(&txn)?);
        }
        Ok(())
    }

    /// Derive a `MerkleTree` instance from the set of pool transactions.
    pub fn merkle_tree(&self) -> VdrResult<MerkleTree> {
        Ok(MerkleTree::from_vec(self.inner.clone())?)
    }

    /// Convert the set of pool transactions into a `MerkleTree` instance.
    pub fn into_merkle_tree(self) -> VdrResult<MerkleTree> {
        Ok(MerkleTree::from_vec(self.inner)?)
    }

    /// Get the root hash corresponding to the transactions
    pub fn root_hash(&self) -> VdrResult<Vec<u8>> {
        Ok(self.merkle_tree()?.root_hash().clone())
    }

    /// Get the root hash corresponding to the transactions
    pub fn root_hash_base58(&self) -> VdrResult<String> {
        Ok(base58::encode(self.merkle_tree()?.root_hash()))
    }

    /// Iterate the set of transactions as a sequence of msgpack-encoded byte strings.
    pub fn iter(&self) -> impl Iterator<Item = &Vec<u8>> {
        self.inner.iter()
    }

    /// Get a sequence of JSON strings representing the pool transactions.
    pub fn encode_json(&self) -> VdrResult<Vec<String>> {
        Ok(self
            .json_values()?
            .into_iter()
            .map(|v| v.to_string())
            .collect())
    }

    /// Get a sequence of JSON strings representing the pool transactions.
    pub fn encode_json_string(&self) -> VdrResult<String> {
        let mut buf = String::new();
        for line in self.json_values()? {
            buf.push_str(&line.to_string());
            buf.push_str(LINE_ENDING);
        }
        Ok(buf)
    }

    /// Get a sequence of `serde_json::Value` instances representing the pool transactions.
    pub fn json_values(&self) -> VdrResult<Vec<SJsonValue>> {
        self.inner.iter().try_fold(vec![], |mut vec, txn| {
            let value = rmp_serde::decode::from_slice(txn)
                .with_input_err("Genesis transaction cannot be decoded")?;
            vec.push(value);
            Ok(vec)
        })
    }

    /// Get the number of pool transactions.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl std::fmt::Debug for PoolTransactions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PoolTransactions(len={})", self.len())
    }
}

impl std::fmt::Display for PoolTransactions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Ok(vec_json) = self.encode_json() else {
            return Err(std::fmt::Error {});
        };
        let txns = SJsonValue::from(vec_json);
        write!(f, "{}", txns)
    }
}

impl From<Vec<Vec<u8>>> for PoolTransactions {
    fn from(txns: Vec<Vec<u8>>) -> Self {
        Self::new(txns)
    }
}

impl From<&MerkleTree> for PoolTransactions {
    fn from(merkle_tree: &MerkleTree) -> Self {
        Self::from_transactions(merkle_tree)
    }
}

impl TryFrom<&[String]> for PoolTransactions {
    type Error = VdrError;

    fn try_from(txns: &[String]) -> VdrResult<Self> {
        PoolTransactions::from_json_transactions(txns)
    }
}

fn _json_to_msgpack(txn: &SJsonValue) -> VdrResult<Vec<u8>> {
    if let Some(txn) = txn.as_object() {
        let mp_txn = rmp_serde::encode::to_vec_named(txn)
            .with_input_err("Can't encode genesis txn as msgpack")?;
        Ok(mp_txn)
    } else {
        Err(input_err("Unexpected value, not a JSON object"))
    }
}

fn _json_iter_to_msgpack<T>(mut iter: T) -> VdrResult<Vec<Vec<u8>>>
where
    T: Iterator<Item = Result<SJsonValue, serde_json::Error>>,
{
    iter.try_fold(vec![], |mut vec, txn| match txn {
        Ok(txn) => {
            let mp_txn = _json_to_msgpack(&txn)?;
            vec.push(mp_txn);
            Ok(vec)
        }
        Err(err) => Err(input_err(format!(
            "Error parsing genesis transactions: {}",
            err
        ))),
    })
}

pub trait PoolTransactionsCache: Send + Sync {
    fn resolve_latest(&self, txns: &PoolTransactions) -> VdrResult<Option<PoolTransactions>>;

    fn update(&self, base: &PoolTransactions, latest: &PoolTransactions) -> VdrResult<()>;
}

#[derive(Debug, Clone)]
struct _MemoryCacheEntry {
    txns: PoolTransactions,
}

#[derive(Debug, Default)]
pub struct InMemoryCache {
    cache: Mutex<HashMap<String, _MemoryCacheEntry>>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PoolTransactionsCache for InMemoryCache {
    fn resolve_latest(&self, txns: &PoolTransactions) -> VdrResult<Option<PoolTransactions>> {
        let hash = txns.root_hash_base58()?;
        let cache = self.cache.lock().unwrap();
        if let Some(entry) = cache.get(&hash) {
            Ok(Some(entry.txns.clone()))
        } else {
            Ok(None)
        }
    }

    fn update(&self, base: &PoolTransactions, latest: &PoolTransactions) -> VdrResult<()> {
        let from_hash = base.root_hash_base58()?;
        let mut cache = self.cache.lock().unwrap();
        cache
            .entry(from_hash)
            .and_modify(|e| e.txns = latest.clone())
            .or_insert_with(|| _MemoryCacheEntry {
                txns: latest.clone(),
            });
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FilesystemCache {
    cache_dir: PathBuf,
}

impl FilesystemCache {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            cache_dir: path.into(),
        }
    }

    fn establish(&self, ident: &str) -> VdrResult<()> {
        Ok(fs::create_dir_all(self.cache_dir.join(ident))?)
    }

    fn read_cache_file(&self, ident: &str, name: &str) -> Option<String> {
        let mut path = self.cache_dir.clone();
        path.extend(&[ident, name]);
        fs::read_to_string(path)
            .map_err(|e| {
                if e.kind() != io::ErrorKind::NotFound {
                    warn!("Error reading from pool genesis cache: {e}")
                }
            })
            .ok()
    }

    fn write_cache_file(&self, ident: &str, name: &str, contents: &str) -> VdrResult<()> {
        self.establish(ident)?;
        let mut target_path = self.cache_dir.clone();
        target_path.push(ident);
        let temp_name = format!("{:020}.tmp", random::<u64>());
        let temp_path = target_path.join(temp_name);
        target_path.push(name);
        fs::write(&temp_path, contents.as_bytes())
            .and_then(|_| fs::rename(&temp_path, &target_path))
            .map_err(|e| warn!("Error writing from pool genesis cache: {e}"))
            .ok();
        Ok(())
    }
}

impl PoolTransactionsCache for FilesystemCache {
    fn resolve_latest(&self, txns: &PoolTransactions) -> VdrResult<Option<PoolTransactions>> {
        let ident = txns.root_hash_base58()?;
        if let Some(txns) = self.read_cache_file(&ident, "txns") {
            Ok(PoolTransactions::from_json(&txns)
                .map_err(|e| warn!("Error reading from pool genesis cache: {e}"))
                .ok())
        } else {
            Ok(None)
        }
    }

    fn update(&self, base: &PoolTransactions, latest: &PoolTransactions) -> VdrResult<()> {
        let ident = base.root_hash_base58()?;
        self.write_cache_file(&ident, "txns", &latest.encode_json_string()?)?;
        Ok(())
    }
}

pub fn build_node_transaction_map<T>(
    txns: T,
    protocol_version: ProtocolVersion,
) -> VdrResult<NodeTransactionMap>
where
    T: IntoIterator,
    T::Item: AsRef<[u8]>,
{
    txns.into_iter().try_fold(HashMap::new(), |mut map, txn| {
        let mut node_txn = _decode_transaction(txn.as_ref(), protocol_version)?;
        let dest = node_txn.txn.data.dest.clone();
        let exist: Option<&mut NodeTransactionV1> = map.get_mut(&dest);
        if exist.is_some() {
            exist.unwrap().update(&mut node_txn)?;
        } else {
            map.insert(dest, node_txn);
        }
        Ok(map)
    })
}

pub fn build_verifiers(txn_map: NodeTransactionMap) -> VdrResult<Verifiers> {
    Ok(txn_map
        .into_iter()
        .map(|(public_key, txn)| {
            let node_alias = txn.txn.data.data.alias.clone();

            if txn.txn.data.data.services.is_none()
                || !txn
                    .txn
                    .data
                    .data
                    .services
                    .as_ref()
                    .unwrap()
                    .contains(&"VALIDATOR".to_string())
            {
                return Err(input_err(format!(
                    "Node '{}' is not a validator",
                    node_alias
                )));
            }

            let verkey = EncodedVerKey::new(
                &public_key,
                Some(KeyType::ED25519),
                Some(KeyEncoding::BASE58),
            )
            .decode()
            .map_input_err(|| {
                format!(
                    "Node '{}' has invalid field 'dest': failed parsing base58",
                    node_alias
                )
            })?;

            let enc_key = verkey
                .key_exchange()
                .map_input_err(|| {
                    format!(
                        "Node '{}' has invalid field 'dest': key not accepted",
                        node_alias
                    )
                })?
                .key_bytes()
                .to_vec();

            let client_addr = match (&txn.txn.data.data.client_ip, &txn.txn.data.data.client_port) {
                (Some(ref client_ip), Some(ref client_port)) => {
                    format!("tcp://{}:{}", client_ip, client_port)
                }
                _ => {
                    return Err(input_err(format!(
                        "Node '{}' has no client address",
                        node_alias
                    )))
                }
            };

            let node_addr = match (&txn.txn.data.data.node_ip, &txn.txn.data.data.node_port) {
                (Some(ref node_ip), Some(ref node_port)) => {
                    format!("tcp://{}:{}", node_ip, node_port)
                }
                _ => {
                    return Err(input_err(format!(
                        "Node '{}' has no node address",
                        node_alias
                    )))
                }
            };

            let bls_key =
                match txn.txn.data.data.blskey {
                    Some(ref blskey) => {
                        let key = base58::decode(blskey.as_str()).map_input_err(|| {
                            format!("Node '{}': invalid base58 in field blskey", node_alias)
                        })?;
                        Some(VerifierKey::from_bytes(&key).map_input_err(|| {
                            format!("Node '{}': invalid field blskey", node_alias)
                        })?)
                    }
                    None => None,
                };

            let info = VerifierInfo {
                client_addr,
                node_addr,
                bls_key,
                public_key,
                enc_key,
            };

            Ok((node_alias, info))
        })
        .fold(HashMap::new(), |mut map, res| {
            match res {
                Err(err) => {
                    info!("Skipped: {}", err);
                }
                Ok((alias, info)) => {
                    map.insert(alias, info);
                }
            }
            map
        }))
}

fn _decode_transaction(
    gen_txn: &[u8],
    protocol_version: ProtocolVersion,
) -> VdrResult<NodeTransactionV1> {
    let gen_txn: NodeTransaction = rmp_serde::decode::from_slice(gen_txn)
        .with_input_err("Genesis transaction cannot be decoded")?;

    match gen_txn {
        NodeTransaction::NodeTransactionV0(txn) => {
            if protocol_version != 1 {
                Err(err_msg(VdrErrorKind::Incompatible,
                            format!("Libindy PROTOCOL_VERSION is {} but Pool Genesis Transactions are of version {}.\
                                           Call indy_set_protocol_version(1) to set correct PROTOCOL_VERSION",
                                    protocol_version, NodeTransactionV0::VERSION)))
            } else {
                Ok(NodeTransactionV1::from(txn))
            }
        }
        NodeTransaction::NodeTransactionV1(txn) => {
            if protocol_version != 2 {
                Err(err_msg(VdrErrorKind::Incompatible,
                            format!("Libindy PROTOCOL_VERSION is {} but Pool Genesis Transactions are of version {}.\
                                           Call indy_set_protocol_version(2) to set correct PROTOCOL_VERSION",
                                    protocol_version, NodeTransactionV1::VERSION)))
            } else {
                Ok(txn)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;

    use super::*;

    const NODE1: &str = r#"{"reqSignature":{},"txn":{"data":{"data":{"alias":"Node1","blskey":"4N8aUNHSgjQVgkpm8nhNEfDf6txHznoYREg9kirmJrkivgL4oSEimFF6nsQ6M41QvhM2Z33nves5vfSn9n1UwNFJBYtWVnHYMATn76vLuL3zU88KyeAYcHfsih3He6UHcXDxcaecHVz6jhCYz1P2UZn2bDVruL5wXpehgBfBaLKm3Ba","blskey_pop":"RahHYiCvoNCtPTrVtP7nMC5eTYrsUA8WjXbdhNc8debh1agE9bGiJxWBXYNFbnJXoXhWFMvyqhqhRoq737YQemH5ik9oL7R4NTTCz2LEZhkgLJzB3QRQqJyBNyv7acbdHrAT8nQ9UkLbaVL9NBpnWXBTw4LEMePaSHEw66RzPNdAX1","client_ip":"127.0.0.1","client_port":9702,"node_ip":"127.0.0.1","node_port":9701,"services":["VALIDATOR"]},"dest":"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv"},"metadata":{"from":"Th7MpTaRZVRYnPiabds81Y"},"type":"0"},"txnMetadata":{"seqNo":1,"txnId":"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62"},"ver":"1"}"#;
    const NODE2: &str = r#"{"reqSignature":{},"txn":{"data":{"data":{"alias":"Node2","blskey":"37rAPpXVoxzKhz7d9gkUe52XuXryuLXoM6P6LbWDB7LSbG62Lsb33sfG7zqS8TK1MXwuCHj1FKNzVpsnafmqLG1vXN88rt38mNFs9TENzm4QHdBzsvCuoBnPH7rpYYDo9DZNJePaDvRvqJKByCabubJz3XXKbEeshzpz4Ma5QYpJqjk","blskey_pop":"Qr658mWZ2YC8JXGXwMDQTzuZCWF7NK9EwxphGmcBvCh6ybUuLxbG65nsX4JvD4SPNtkJ2w9ug1yLTj6fgmuDg41TgECXjLCij3RMsV8CwewBVgVN67wsA45DFWvqvLtu4rjNnE9JbdFTc1Z4WCPA3Xan44K1HoHAq9EVeaRYs8zoF5","client_ip":"127.0.0.1","client_port":9704,"node_ip":"127.0.0.1","node_port":9703,"services":["VALIDATOR"]},"dest":"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"},"metadata":{"from":"EbP4aYNeTHL6q385GuVpRV"},"type":"0"},"txnMetadata":{"seqNo":2,"txnId":"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc"},"ver":"1"}"#;
    const NODE3: &str = r#"{"reqSignature":{},"txn":{"data":{"data":{"alias":"Node3","blskey":"3WFpdbg7C5cnLYZwFZevJqhubkFALBfCBBok15GdrKMUhUjGsk3jV6QKj6MZgEubF7oqCafxNdkm7eswgA4sdKTRc82tLGzZBd6vNqU8dupzup6uYUf32KTHTPQbuUM8Yk4QFXjEf2Usu2TJcNkdgpyeUSX42u5LqdDDpNSWUK5deC5","blskey_pop":"QwDeb2CkNSx6r8QC8vGQK3GRv7Yndn84TGNijX8YXHPiagXajyfTjoR87rXUu4G4QLk2cF8NNyqWiYMus1623dELWwx57rLCFqGh7N4ZRbGDRP4fnVcaKg1BcUxQ866Ven4gw8y4N56S5HzxXNBZtLYmhGHvDtk6PFkFwCvxYrNYjh","client_ip":"127.0.0.1","client_port":9706,"node_ip":"127.0.0.1","node_port":9705,"services":["VALIDATOR"]},"dest":"DKVxG2fXXTU8yT5N7hGEbXB3dfdAnYv1JczDUHpmDxya"},"metadata":{"from":"4cU41vWW82ArfxJxHkzXPG"},"type":"0"},"txnMetadata":{"seqNo":3,"txnId":"7e9f355dffa78ed24668f0e0e369fd8c224076571c51e2ea8be5f26479edebe4"},"ver":"1"}"#;
    const NODE1_OLD: &str = r#"{"data":{"alias":"Node1","client_ip":"192.168.1.35","client_port":9702,"node_ip":"192.168.1.35","node_port":9701,"services":["VALIDATOR"]},"dest":"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv","identifier":"FYmoFw55GeQH7SRFa37dkx1d2dZ3zUF8ckg7wmL7ofN4","txnId":"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62","type":"0"}"#;
    const NODE2_OLD: &str = r#"{"data":{"alias":"Node2","client_ip":"192.168.1.35","client_port":9704,"node_ip":"192.168.1.35","node_port":9703,"services":["VALIDATOR"]},"dest":"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb","identifier":"8QhFxKxyaFsJy4CyxeYX34dFH8oWqyBv1P4HLQCsoeLy","txnId":"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc","type":"0"}"#;

    fn _merkle_tree() -> MerkleTree {
        _transactions().merkle_tree().unwrap()
    }

    fn _transactions() -> PoolTransactions {
        PoolTransactions::from_json_transactions(&[NODE1, NODE2, NODE3]).unwrap()
    }

    #[test]
    fn test_build_node_transaction_map_works_for_node_1_4_and_protocol_version_1_4() {
        let txn_map = build_node_transaction_map(_merkle_tree(), ProtocolVersion::Node1_4).unwrap();

        assert_eq!(3, txn_map.len());
        assert!(txn_map.contains_key("Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv"));
        assert!(txn_map.contains_key("8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"));

        let node1: NodeTransactionV1 = serde_json::from_str(NODE1).unwrap();
        let node2: NodeTransactionV1 = serde_json::from_str(NODE2).unwrap();

        assert_eq!(
            txn_map["Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv"],
            node1
        );
        assert_eq!(
            txn_map["8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"],
            node2
        );
    }

    #[test]
    fn test_build_node_transaction_map_works_for_node_1_4_and_protocol_version_1_3() {
        let _err =
            build_node_transaction_map(_merkle_tree(), ProtocolVersion::Node1_3).unwrap_err();
    }

    #[test]
    fn test_build_node_transaction_map_works_for_node_1_3_and_protocol_version_1_3() {
        let merkle_tree = PoolTransactions::from_json_transactions(&[NODE1_OLD, NODE2_OLD])
            .unwrap()
            .merkle_tree()
            .unwrap();

        let txn_map = build_node_transaction_map(merkle_tree, ProtocolVersion::Node1_3).unwrap();

        assert_eq!(2, txn_map.len());
        assert!(txn_map.contains_key("Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv"));
        assert!(txn_map.contains_key("8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"));

        let node1: NodeTransactionV1 =
            NodeTransactionV1::from(serde_json::from_str::<NodeTransactionV0>(NODE1_OLD).unwrap());
        let node2: NodeTransactionV1 =
            NodeTransactionV1::from(serde_json::from_str::<NodeTransactionV0>(NODE2_OLD).unwrap());

        assert_eq!(
            txn_map["Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv"],
            node1
        );
        assert_eq!(
            txn_map["8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"],
            node2
        );
    }

    #[test]
    fn test_build_node_transaction_map_works_for_node_1_3_and_protocol_version_1_4() {
        let merkle_tree = PoolTransactions::from_json_transactions(&[NODE1_OLD, NODE2_OLD])
            .unwrap()
            .merkle_tree()
            .unwrap();

        let _err = build_node_transaction_map(merkle_tree, ProtocolVersion::Node1_4).unwrap_err();
    }

    #[test]
    fn test_in_memory_cache() {
        let txns = _transactions();
        let cache = InMemoryCache::new();
        assert_eq!(cache.resolve_latest(&txns).unwrap(), None);
        let mut txns_long = txns.clone();
        txns_long.extend_from_json(&[NODE1_OLD]).unwrap();
        cache.update(&txns, &txns_long).unwrap();
        assert_eq!(
            cache.resolve_latest(&txns).unwrap().as_ref(),
            Some(&txns_long)
        );
        txns_long.extend_from_json(&[NODE2_OLD]).unwrap();
        cache.update(&txns, &txns_long).unwrap();
        assert_eq!(
            cache.resolve_latest(&txns).unwrap().as_ref(),
            Some(&txns_long)
        );
    }

    #[test]
    fn test_fs_cache() {
        let temp_name = format!("vdr-test-{:020}", random::<u64>());
        let temp_dir = temp_dir().join(temp_name);
        let txns = _transactions();
        let cache = FilesystemCache::new(&temp_dir);
        assert_eq!(cache.resolve_latest(&txns).unwrap(), None);
        let mut txns_long = txns.clone();
        txns_long.extend_from_json(&[NODE1_OLD]).unwrap();
        cache.update(&txns, &txns_long).unwrap();
        assert_eq!(
            cache.resolve_latest(&txns).unwrap().as_ref(),
            Some(&txns_long)
        );
        txns_long.extend_from_json(&[NODE2_OLD]).unwrap();
        cache.update(&txns, &txns_long).unwrap();
        assert_eq!(
            cache.resolve_latest(&txns).unwrap().as_ref(),
            Some(&txns_long)
        );
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
