#[macro_use]
mod utils;

inject_dependencies!();

use indy_vdr::ledger::constants;
use indy_vdr::utils::did::DidValue;

use crate::utils::fixtures::*;

#[test]
fn empty() {
    // Empty test to run module
}

#[cfg(test)]
mod builder {
    use super::*;
    use indy_vdr::ledger::RequestBuilder;

    mod freeze_ledger {
        use super::*;
        use crate::utils::helpers::check_request_operation;

        #[rstest]
        fn test_freeze_ledger_request(request_builder: RequestBuilder, trustee_did: DidValue) {
            let ledgers_ids = vec![0, 1, 50, 873];

            let request = request_builder
                .build_ledgers_freeze_request(&trustee_did, &ledgers_ids)
                .unwrap();

            let expected_operation = json!({
                "type": constants::LEDGERS_FREEZE,
                "ledgers_ids": ledgers_ids
            });
            check_request_operation(&request, expected_operation);
        }
    }

    mod get_frozen_ledgers {
        use super::*;
        use crate::utils::helpers::check_request_operation;

        #[rstest]
        fn test_get_frozen_ledgers_request(request_builder: RequestBuilder, trustee_did: DidValue) {
            let request = request_builder
                .build_get_frozen_ledgers_request(&trustee_did)
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_FROZEN_LEDGERS,
            });
            check_request_operation(&request, expected_operation);
        }
    }
}
