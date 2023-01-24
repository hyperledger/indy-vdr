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

fn _datetime() -> String {
    format!(
        "{}-01-25T12:49:05.258870+00:00",
        time::OffsetDateTime::now_utc().year() + 1
    )
}

#[cfg(test)]
mod builder {
    use super::*;
    use indy_vdr::ledger::RequestBuilder;

    mod pool_config {
        use super::*;
        use crate::utils::helpers::check_request_operation;

        #[rstest]
        fn test_build_pool_restart_request(request_builder: RequestBuilder, trustee_did: DidValue) {
            let request = request_builder
                .build_pool_restart_request(&trustee_did, "start", None)
                .unwrap();

            let expected_operation = json!({
                "type": constants::POOL_RESTART,
                "action": "start",
            });
            check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_pool_restart_request_for_cancel(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_pool_restart_request(&trustee_did, "cancel", None)
                .unwrap();

            let expected_operation = json!({
                "type": constants::POOL_RESTART,
                "action": "cancel"
            });
            check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_pool_restart_request_for_datetime(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_pool_restart_request(&trustee_did, "start", Some(&_datetime()))
                .unwrap();

            let expected_operation = json!({
                "type": constants::POOL_RESTART,
                "action": "start",
                "datetime": _datetime()
            });
            check_request_operation(&request, expected_operation);
        }
    }
}

#[cfg(test)]
#[cfg(feature = "local_nodes_pool")]
mod send_pool_restart {
    use super::*;
    use crate::utils::crypto::Identity;
    use crate::utils::helpers;
    use crate::utils::pool::TestPool;

    #[rstest]
    fn test_pool_send_pool_restart_request(pool: TestPool, trustee: Identity) {
        // Start Pool Restart
        let mut request = pool
            .request_builder()
            .build_pool_restart_request(&trustee.did, "start", Some(&_datetime()))
            .unwrap();

        let _response = helpers::sign_and_send_request(&trustee, &pool, &mut request).unwrap();

        // Cancel Pool Restart
        let mut request = pool
            .request_builder()
            .build_pool_restart_request(&trustee.did, "cancel", None)
            .unwrap();

        let _response = helpers::sign_and_send_request(&trustee, &pool, &mut request).unwrap();
    }
}
