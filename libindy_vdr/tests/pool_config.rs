#[macro_use]
mod utils;

inject_dependencies!();

use indy_vdr::common::did::DidValue;
use indy_vdr::ledger::constants;

use crate::utils::crypto::Identity;
use crate::utils::fixtures::*;
use crate::utils::pool::TestPool;

#[test]
fn empty() {
    // Empty test to run module
}

#[cfg(test)]
mod builder {
    use super::*;
    use indy_vdr::ledger::RequestBuilder;

    mod pool_config {
        use super::*;
        use crate::utils::helpers::check_request_operation;

        #[rstest]
        fn test_pool_config_request(request_builder: RequestBuilder, trustee_did: DidValue) {
            let request = request_builder
                .build_pool_config(&trustee_did, true, false)
                .unwrap();

            let expected_operation = json!({
                "type": constants::POOL_CONFIG,
                "writes": true,
                "force": false
            });
            check_request_operation(&request, expected_operation);
        }
    }
}

#[cfg(test)]
mod send_pool_config {
    use super::*;
    use crate::utils::helpers;

    #[rstest]
    fn test_pool_send_pool_config_request(pool: TestPool, trustee: Identity) {
        // Send Pool Config
        let mut request = pool
            .request_builder()
            .build_pool_config(&trustee.did, true, false)
            .unwrap();

        let _response = helpers::sign_and_send_request(&trustee, &pool, &mut request).unwrap();
    }
}
