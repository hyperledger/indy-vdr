#[macro_use]
mod utils;

use indy_vdr::common::merkle_tree::MerkleTree;
use indy_vdr::pool::PoolTransactions;

use utils::pool::default_transactions;

mod pool_transactions_tests {
    use super::*;

    pub fn _merkle_tree() -> MerkleTree {
        PoolTransactions::from_json_transactions(default_transactions())
            .unwrap()
            .merkle_tree()
            .unwrap()
    }

    #[test]
    fn test_pool_transactions_from_transactions_json_works() {
        let txns = default_transactions();

        let transactions: PoolTransactions =
            PoolTransactions::from_json_transactions(&txns).unwrap();

        assert_eq!(transactions.encode_json().unwrap(), txns)
    }

    // #[test]
    // fn test_pool_transactions_from_file_works() {
    //     let mut transaction = GenesisTransactions::new(None);
    //     let file = transaction.store_to_file();

    //     let transactions: PoolTransactions = PoolTransactions::from_json_file(file).unwrap();

    //     assert_eq!(
    //         transactions.encode_json().unwrap(),
    //         GenesisTransactions::default_transactions()
    //     )
    // }

    // #[test]
    // fn test_pool_transactions_from_file_for_unknown_file() {
    //     let file = {
    //         let mut transaction = GenesisTransactions::new(None);
    //         transaction.store_to_file()
    //     };

    //     let _err = PoolTransactions::from_json_file(file).unwrap_err();
    // }

    // #[test]
    // fn test_pool_transactions_from_file_for_invalid_transactions() {
    //     let mut txns = GenesisTransactions::from_transactions([r#"{invalid}"#]);
    //     let _err = PoolTransactions::from_json_file(txns.store_to_file()).unwrap_err();
    // }

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
