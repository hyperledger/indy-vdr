use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;
use std::iter::IntoIterator;
use std::path::Path;

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
                VdrErrorKind::FileSystem(err),
                format!("Can't open genesis transactions file: {:?}", file_name),
            )
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
        let vec_json = unwrap_or_return!(self.encode_json(), Err(std::fmt::Error {}));
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
                (&Some(ref client_ip), &Some(ref client_port)) => {
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
                (&Some(ref node_ip), &Some(ref node_port)) => {
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
    use super::*;

    mod pool_transactions_tests {
        use super::*;
        use indy_test_utils::genesis::GenesisTransactions;

        pub fn _transactions() -> Vec<String> {
            GenesisTransactions::new(Some(4)).transactions
        }

        pub fn _merkle_tree() -> MerkleTree {
            PoolTransactions::from_json_transactions(&_transactions())
                .unwrap()
                .merkle_tree()
                .unwrap()
        }

        #[test]
        fn test_pool_transactions_from_transactions_json_works() {
            let transaction = GenesisTransactions::new(None);

            let transactions: PoolTransactions =
                PoolTransactions::from_json_transactions(&transaction.transactions).unwrap();

            assert_eq!(
                transactions.encode_json().unwrap(),
                GenesisTransactions::default_transactions()
            )
        }

        #[test]
        fn test_pool_transactions_from_file_works() {
            let mut transaction = GenesisTransactions::new(None);
            let file = transaction.store_to_file();

            let transactions: PoolTransactions = PoolTransactions::from_json_file(&file).unwrap();

            assert_eq!(
                transactions.encode_json().unwrap(),
                GenesisTransactions::default_transactions()
            )
        }

        #[test]
        fn test_pool_transactions_from_file_for_unknown_file() {
            let file = {
                let mut transaction = GenesisTransactions::new(None);
                transaction.store_to_file()
            };

            let _err = PoolTransactions::from_json_file(&file).unwrap_err();
        }

        #[test]
        fn test_pool_transactions_from_file_for_invalid_transactions() {
            let mut txns = GenesisTransactions::from_transactions(&[r#"{invalid}"#]);
            let _err = PoolTransactions::from_json_file(txns.store_to_file()).unwrap_err();
        }

        #[test]
        fn test_merkle_tree_from_transactions_works() {
            let merkle_tree = _merkle_tree();

            assert_eq!(merkle_tree.count(), 4, "test restored MT size");
            assert_eq!(
                merkle_tree.root_hash_hex(),
                "ef25b5d33e511d2b8e3fbf267cc4496a77cf522976d5ac158878f787190d9a97",
                "test restored MT root hash"
            );
        }
    }

    mod build_node_transaction_map_tests {
        use super::*;
        use crate::pool::genesis::tests::pool_transactions_tests::{_merkle_tree, _transactions};

        pub const NODE1_OLD: &str = r#"{"data":{"alias":"Node1","client_ip":"192.168.1.35","client_port":9702,"node_ip":"192.168.1.35","node_port":9701,"services":["VALIDATOR"]},"dest":"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv","identifier":"FYmoFw55GeQH7SRFa37dkx1d2dZ3zUF8ckg7wmL7ofN4","txnId":"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62","type":"0"}"#;
        pub const NODE2_OLD: &str = r#"{"data":{"alias":"Node2","client_ip":"192.168.1.35","client_port":9704,"node_ip":"192.168.1.35","node_port":9703,"services":["VALIDATOR"]},"dest":"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb","identifier":"8QhFxKxyaFsJy4CyxeYX34dFH8oWqyBv1P4HLQCsoeLy","txnId":"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc","type":"0"}"#;

        #[test]
        fn test_build_node_transaction_map_works_for_node_1_4_and_protocol_version_1_4() {
            let txn_map =
                build_node_transaction_map(_merkle_tree(), ProtocolVersion::Node1_4).unwrap();

            assert_eq!(4, txn_map.len());
            assert!(txn_map.contains_key("Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv"));
            assert!(txn_map.contains_key("8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"));

            let node1: NodeTransactionV1 = serde_json::from_str(&_transactions()[0]).unwrap();
            let node2: NodeTransactionV1 = serde_json::from_str(&_transactions()[1]).unwrap();

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
            let merkle_tree = PoolTransactions::from_json_transactions(vec![NODE1_OLD, NODE2_OLD])
                .unwrap()
                .merkle_tree()
                .unwrap();

            let txn_map =
                build_node_transaction_map(merkle_tree, ProtocolVersion::Node1_3).unwrap();

            assert_eq!(2, txn_map.len());
            assert!(txn_map.contains_key("Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv"));
            assert!(txn_map.contains_key("8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"));

            let node1: NodeTransactionV1 = NodeTransactionV1::from(
                serde_json::from_str::<NodeTransactionV0>(NODE1_OLD).unwrap(),
            );
            let node2: NodeTransactionV1 = NodeTransactionV1::from(
                serde_json::from_str::<NodeTransactionV0>(NODE2_OLD).unwrap(),
            );

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
            let merkle_tree = PoolTransactions::from_json_transactions(vec![NODE1_OLD, NODE2_OLD])
                .unwrap()
                .merkle_tree()
                .unwrap();

            let _err =
                build_node_transaction_map(merkle_tree, ProtocolVersion::Node1_4).unwrap_err();
        }
    }
}
