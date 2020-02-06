use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;
use std::iter::IntoIterator;
use std::path::PathBuf;

use serde_json::{self, Deserializer, Value as SJsonValue};

use super::types::{
    BlsVerKey, NodeTransaction, NodeTransactionV0, NodeTransactionV1, ProtocolVersion,
    VerifierInfo, Verifiers,
};
use crate::common::error::prelude::*;
use crate::common::merkle_tree::MerkleTree;
use crate::utils::base58::FromBase58;
use crate::utils::crypto;

pub type NodeTransactionMap = HashMap<String, NodeTransactionV1>;

#[derive(Clone, PartialEq, Eq)]
pub struct PoolTransactions {
    inner: Vec<Vec<u8>>, // stored in msgpack format
}

impl PoolTransactions {
    fn new(inner: Vec<Vec<u8>>) -> Self {
        Self { inner }
    }

    pub fn from_file(file_name: &str) -> VdrResult<Self> {
        Self::from_file_path(&PathBuf::from(file_name))
    }

    pub fn from_file_path(file_name: &PathBuf) -> VdrResult<Self> {
        let f = File::open(file_name).map_err(|err| {
            err_msg(
                VdrErrorKind::FileSystem(err),
                format!("Can't open genesis transactions file: {:?}", file_name),
            )
        })?;
        let reader = BufReader::new(&f);
        let mut stream = Deserializer::from_reader(reader).into_iter::<SJsonValue>();
        let txns = stream.try_fold(vec![], |mut vec, txn| match txn {
            Ok(txn) => {
                let mp_txn = _json_to_msgpack(&txn)?;
                vec.push(mp_txn);
                Ok(vec)
            }
            Err(err) => Err(input_err(format!(
                "Error reading from genesis transactions file: {}",
                err
            ))),
        })?;
        if txns.is_empty() {
            Err(input_err("No genesis transactions found"))
        } else {
            Ok(Self::new(txns))
        }
    }

    pub fn from_transactions<T>(txns: T) -> Self
    where
        T: IntoIterator,
        T::Item: AsRef<[u8]>,
    {
        Self::new(txns.into_iter().map(|t| t.as_ref().to_vec()).collect())
    }

    pub fn from_json<T>(txns: T) -> VdrResult<Self>
    where
        T: IntoIterator,
        T::Item: AsRef<str>,
    {
        let mut pt = Self { inner: vec![] };
        pt.extend_from_json(txns)?;
        Ok(pt)
    }

    pub fn extend<T>(&mut self, txns: T)
    where
        T: IntoIterator<Item = Vec<u8>>,
    {
        self.inner.extend(txns)
    }

    pub fn extend_from_json<'a, T>(&mut self, txns: T) -> VdrResult<()>
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

    pub fn merkle_tree(&self) -> VdrResult<MerkleTree> {
        MerkleTree::from_vec(self.inner.clone())
    }

    pub fn into_merkle_tree(self) -> VdrResult<MerkleTree> {
        MerkleTree::from_vec(self.inner)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Vec<u8>> {
        self.inner.iter()
    }

    pub fn encode_json(&self) -> VdrResult<Vec<String>> {
        Ok(self
            .json_values()?
            .into_iter()
            .map(|v| v.to_string())
            .collect())
    }

    pub fn json_values(&self) -> VdrResult<Vec<SJsonValue>> {
        self.inner.iter().try_fold(vec![], |mut vec, txn| {
            let value = rmp_serde::decode::from_slice(txn)
                .with_input_err("Genesis transaction cannot be decoded")?;
            vec.push(value);
            Ok(vec)
        })
    }

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
        PoolTransactions::from_json(txns)
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

            let verkey_bin = public_key.from_base58().map_input_err(|| {
                format!(
                    "Node '{}' has invalid field 'dest': failed parsing base58",
                    node_alias
                )
            })?;

            let enc_key = crypto::import_verkey(&verkey_bin)
                .and_then(|vk| crypto::vk_to_curve25519(vk))
                .map_input_err(|| {
                    format!(
                        "Node '{}' has invalid field 'dest': key not accepted",
                        node_alias
                    )
                })?;

            let address = match (&txn.txn.data.data.client_ip, &txn.txn.data.data.client_port) {
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

            let bls_key: Option<BlsVerKey> = match txn.txn.data.data.blskey {
                Some(ref blskey) => {
                    let key = blskey.as_str().from_base58().map_input_err(|| {
                        format!("Node '{}': invalid base58 in field blskey", node_alias)
                    })?;

                    Some(BlsVerKey::from_bytes(&key).map_err(|_| {
                        input_err(format!("Node '{}': invalid field blskey", node_alias))
                    })?)
                }
                None => None,
            };

            let info = VerifierInfo {
                address,
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
        })
        .into())
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
                Err(input_err(
                    format!("Libindy PROTOCOL_VERSION is {} but Pool Genesis Transactions are of version {}.\
                            Call indy_set_protocol_version(1) to set correct PROTOCOL_VERSION",
                            protocol_version, NodeTransactionV0::VERSION)))
            } else {
                Ok(NodeTransactionV1::from(txn))
            }
        }
        NodeTransaction::NodeTransactionV1(txn) => {
            if protocol_version != 2 {
                Err(input_err(
                    format!("Libindy PROTOCOL_VERSION is {} but Pool Genesis Transactions are of version {}.\
                                Call indy_set_protocol_version(2) to set correct PROTOCOL_VERSION",
                            protocol_version, NodeTransactionV1::VERSION)))
            } else {
                Ok(txn)
            }
        }
    }
}

/*
FIXME

#[cfg(test)]
mod tests {
    use std::fs;

    use byteorder::LittleEndian;

    use crate::domain::pool::ProtocolVersion;
    use crate::utils::test;

    use super::*;

    fn _set_protocol_version(version: usize) {
        ProtocolVersion::set(version);
    }

    const TEST_PROTOCOL_VERSION: usize = 2;
    pub const NODE1_OLD: &str = r#"{"data":{"alias":"Node1","client_ip":"192.168.1.35","client_port":9702,"node_ip":"192.168.1.35","node_port":9701,"services":["VALIDATOR"]},"dest":"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv","identifier":"FYmoFw55GeQH7SRFa37dkx1d2dZ3zUF8ckg7wmL7ofN4","txnId":"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62","type":"0"}"#;
    pub const NODE2_OLD: &str = r#"{"data":{"alias":"Node2","client_ip":"192.168.1.35","client_port":9704,"node_ip":"192.168.1.35","node_port":9703,"services":["VALIDATOR"]},"dest":"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb","identifier":"8QhFxKxyaFsJy4CyxeYX34dFH8oWqyBv1P4HLQCsoeLy","txnId":"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc","type":"0"}"#;

    fn _write_genesis_txns(pool_name: &str, txns: &str) {
        let path = get_pool_stored_path_base(pool_name, true, pool_name, POOL_EXT);
        let mut f = fs::File::create(path.as_path()).unwrap();
        f.write(txns.as_bytes()).unwrap();
        f.flush().unwrap();
        f.sync_all().unwrap();
    }

    #[test]
    fn pool_worker_build_node_state_works_for_new_txns_format_and_1_protocol_version() {
        test::cleanup_storage(
            "pool_worker_build_node_state_works_for_new_txns_format_and_1_protocol_version",
        );

        _set_protocol_version(1);

        let node_txns = test::gen_txns();
        let txns_src = node_txns[0..(2 as usize)].join("\n");

        _write_genesis_txns(
            "pool_worker_build_node_state_works_for_new_txns_format_and_1_protocol_version",
            &txns_src,
        );

        let merkle_tree = super::create(
            "pool_worker_build_node_state_works_for_new_txns_format_and_1_protocol_version",
        )
        .unwrap();
        let res = super::build_node_state(&merkle_tree);
        assert_kind!(VdrErrorKind::PoolIncompatibleProtocolVersion, res);

        test::cleanup_storage(
            "pool_worker_build_node_state_works_for_new_txns_format_and_1_protocol_version",
        );
    }

    #[test]
    pub fn pool_worker_works_for_deserialize_cache() {
        test::cleanup_storage("pool_worker_works_for_deserialize_cache");
        {
            _set_protocol_version(TEST_PROTOCOL_VERSION);

            let node_txns = test::gen_txns();

            let txn1_json: serde_json::Value = serde_json::from_str(&node_txns[0]).unwrap();
            let txn2_json: serde_json::Value = serde_json::from_str(&node_txns[1]).unwrap();
            let txn3_json: serde_json::Value = serde_json::from_str(&node_txns[2]).unwrap();
            let txn4_json: serde_json::Value = serde_json::from_str(&node_txns[3]).unwrap();

            let pool_cache = vec![
                rmp_serde::to_vec_named(&txn1_json).unwrap(),
                rmp_serde::to_vec_named(&txn2_json).unwrap(),
                rmp_serde::to_vec_named(&txn3_json).unwrap(),
                rmp_serde::to_vec_named(&txn4_json).unwrap(),
            ];

            let pool_name = "pool_worker_works_for_deserialize_cache";
            let path = get_pool_stored_path(pool_name, true);
            let mut f = fs::File::create(path.as_path()).unwrap();
            pool_cache.iter().for_each(|vec| {
                f.write_u64::<LittleEndian>(vec.len() as u64).unwrap();
                f.write_all(vec).unwrap();
            });

            let merkle_tree = super::create(pool_name).unwrap();
            let _node_state = super::build_node_state(&merkle_tree).unwrap();
        }
        test::cleanup_storage("pool_worker_works_for_deserialize_cache");
    }

    #[test]
    fn pool_worker_restore_merkle_tree_works_from_genesis_txns() {
        test::cleanup_storage("pool_worker_restore_merkle_tree_works_from_genesis_txns");

        let node_txns = test::gen_txns();
        let txns_src = format!(
            "{}\n{}",
            node_txns[0].replace(environment::test_pool_ip().as_str(), "10.0.0.2"),
            node_txns[1].replace(environment::test_pool_ip().as_str(), "10.0.0.2")
        );
        _write_genesis_txns(
            "pool_worker_restore_merkle_tree_works_from_genesis_txns",
            &txns_src,
        );

        let merkle_tree =
            super::create("pool_worker_restore_merkle_tree_works_from_genesis_txns").unwrap();

        assert_eq!(merkle_tree.count(), 2, "test restored MT size");
        assert_eq!(
            merkle_tree.root_hash_hex(),
            "c715aef44aaacab8746c9a505ba106b5554fe6d29ec7f0a2abc9d7723fdea523",
            "test restored MT root hash"
        );

        test::cleanup_storage("pool_worker_restore_merkle_tree_works_from_genesis_txns");
    }

    #[test]
    fn pool_worker_build_node_state_works_for_old_format() {
        test::cleanup_storage("pool_worker_build_node_state_works_for_old_format");

        _set_protocol_version(1);

        let node1: NodeTransactionV1 =
            NodeTransactionV1::from(serde_json::from_str::<NodeTransactionV0>(NODE1_OLD).unwrap());
        let node2: NodeTransactionV1 =
            NodeTransactionV1::from(serde_json::from_str::<NodeTransactionV0>(NODE2_OLD).unwrap());

        let txns_src = format!("{}\n{}\n", NODE1_OLD, NODE2_OLD);

        _write_genesis_txns(
            "pool_worker_build_node_state_works_for_old_format",
            &txns_src,
        );

        let merkle_tree =
            super::create("pool_worker_build_node_state_works_for_old_format").unwrap();
        let node_state = super::build_node_state(&merkle_tree).unwrap();

        assert_eq!(1, ProtocolVersion::get());

        assert_eq!(2, node_state.len());
        assert!(node_state.contains_key("Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv"));
        assert!(node_state.contains_key("8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"));

        assert_eq!(
            node_state["Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv"],
            node1
        );
        assert_eq!(
            node_state["8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"],
            node2
        );

        test::cleanup_storage("pool_worker_build_node_state_works_for_old_format");
    }

    #[test]
    fn pool_worker_build_node_state_works_for_new_format() {
        test::cleanup_storage("pool_worker_build_node_state_works_for_new_format");

        _set_protocol_version(TEST_PROTOCOL_VERSION);

        let node_txns = test::gen_txns();

        let node1: NodeTransactionV1 = serde_json::from_str(&node_txns[0]).unwrap();
        let node2: NodeTransactionV1 = serde_json::from_str(&node_txns[1]).unwrap();

        let txns_src = node_txns.join("\n");

        _write_genesis_txns(
            "pool_worker_build_node_state_works_for_new_format",
            &txns_src,
        );

        let merkle_tree =
            super::create("pool_worker_build_node_state_works_for_new_format").unwrap();
        let node_state = super::build_node_state(&merkle_tree).unwrap();

        assert_eq!(2, ProtocolVersion::get());

        assert_eq!(4, node_state.len());
        assert!(node_state.contains_key("Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv"));
        assert!(node_state.contains_key("8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"));

        assert_eq!(
            node_state["Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv"],
            node1
        );
        assert_eq!(
            node_state["8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"],
            node2
        );

        test::cleanup_storage("pool_worker_build_node_state_works_for_new_format");
    }

    #[test]
    fn pool_worker_build_node_state_works_for_old_txns_format_and_2_protocol_version() {
        test::cleanup_storage(
            "pool_worker_build_node_state_works_for_old_txns_format_and_2_protocol_version",
        );

        _set_protocol_version(TEST_PROTOCOL_VERSION);

        let txns_src = format!("{}\n{}\n", NODE1_OLD, NODE2_OLD);

        _write_genesis_txns(
            "pool_worker_build_node_state_works_for_old_txns_format_and_2_protocol_version",
            &txns_src,
        );

        let merkle_tree = super::create(
            "pool_worker_build_node_state_works_for_old_txns_format_and_2_protocol_version",
        )
        .unwrap();
        let res = super::build_node_state(&merkle_tree);
        assert_kind!(VdrErrorKind::PoolIncompatibleProtocolVersion, res);

        test::cleanup_storage(
            "pool_worker_build_node_state_works_for_old_txns_format_and_2_protocol_version",
        );
    }
}
*/
