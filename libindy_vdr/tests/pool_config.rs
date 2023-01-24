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

    mod pool_config {
        use super::*;
        use crate::utils::helpers::check_request_operation;

        #[rstest]
        fn test_pool_config_request(request_builder: RequestBuilder, trustee_did: DidValue) {
            let request = request_builder
                .build_pool_config_request(&trustee_did, true, false)
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
#[cfg(feature = "local_nodes_pool")]
mod send_pool_config {
    use super::*;
    use crate::utils::crypto::Identity;
    use crate::utils::helpers;
    use crate::utils::pool::TestPool;

    #[rstest]
    fn test_pool_send_pool_config_request_for_disable_writing(pool: TestPool, trustee: Identity) {
        // Send Pool Config
        let mut request = pool
            .request_builder()
            .build_pool_config_request(&trustee.did, false, false)
            .unwrap();

        let _response = helpers::sign_and_send_request(&trustee, &pool, &mut request).unwrap();

        // Try to write Schema
        let (_schema_id, mut schema_request) =
            helpers::schema::build_schema_request(&pool, &trustee);
        let err = helpers::sign_and_send_request(&trustee, &pool, &mut schema_request).unwrap_err();
        helpers::check_response_type(&err, "REQNACK");

        // reset Pool Config
        let mut request = pool
            .request_builder()
            .build_pool_config_request(&trustee.did, true, false)
            .unwrap();

        let _response = helpers::sign_and_send_request(&trustee, &pool, &mut request).unwrap();
    }
}
