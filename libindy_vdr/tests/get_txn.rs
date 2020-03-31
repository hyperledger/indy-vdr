#[macro_use]
mod utils;

inject_dependencies!();

use indy_vdr::ledger::constants;

use crate::utils::fixtures::*;

#[test]
fn empty() {
    // Empty test to run module
}

const SEQ_NO: i32 = 2;
const LEDGER_ID: i32 = 1;

#[cfg(test)]
mod builder {
    use super::*;
    use indy_vdr::ledger::RequestBuilder;

    mod get_txn {
        use super::*;
        use crate::utils::helpers::check_request_operation;

        #[rstest]
        fn test_build_get_txn_request(request_builder: RequestBuilder) {
            let request = request_builder
                .build_get_txn_request(None, LEDGER_ID, SEQ_NO)
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_TXN,
                "data": SEQ_NO,
                "ledgerId": LEDGER_ID
            });

            check_request_operation(&request, expected_operation);
        }
    }
}

#[cfg(test)]
#[cfg(feature = "local_nodes_pool")]
mod get_txn {
    use super::*;
    use crate::utils::crypto::Identity;
    use crate::utils::helpers;
    use crate::utils::pool::TestPool;

    #[rstest]
    fn test_pool_get_txn(pool: TestPool, trustee: Identity, identity: Identity) {
        // Send NYM
        let mut nym_request = pool
            .request_builder()
            .build_nym_request(
                &trustee.did,
                &identity.did,
                Some(identity.verkey.to_string()),
                None,
                None,
            )
            .unwrap();

        let nym_response =
            helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap();

        let seq_no = TestPool::extract_seq_no_from_reply(&nym_response).unwrap();

        std::thread::sleep(std::time::Duration::from_secs(1));

        // Get NYM txn by seq_no
        let get_txn_request = pool
            .request_builder()
            .build_get_txn_request(None, LEDGER_ID, seq_no as i32)
            .unwrap();

        let response = pool.send_request(&get_txn_request).unwrap();

        let nym_response = serde_json::from_str::<serde_json::Value>(&nym_response).unwrap();

        assert_eq!(
            nym_response["result"]["txn"],
            helpers::get_response_data(&response).unwrap()["txn"]
        )
    }

    #[rstest]
    fn test_pool_get_txn_for_unknown_transaction(pool: TestPool) {
        // Get txn by invalid seq_no
        let get_txn_request = pool
            .request_builder()
            .build_get_txn_request(None, LEDGER_ID, i32::max_value())
            .unwrap();

        let response = pool.send_request(&get_txn_request).unwrap();
        helpers::get_response_data(&response).unwrap_err();
    }
}
