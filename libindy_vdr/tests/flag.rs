#[macro_use]
mod utils;

inject_dependencies!();

use indy_vdr::ledger::constants;
use indy_vdr::utils::did::DidValue;

use crate::utils::fixtures::*;
use crate::utils::helpers;

const FLAG_NAME: &str = r#"REV_STRATEGY_USE_COMPAT_ORDERING"#;
const FLAG_VALUE: &str = r#"True"#;

#[test]
fn empty() {
    // Empty test to run module
}

#[cfg(test)]
mod builder {
    use super::*;
    use indy_vdr::ledger::RequestBuilder;

    mod flag {
        use super::*;

        #[rstest]
        fn test_pool_build_flag(request_builder: RequestBuilder, trustee_did: DidValue) {
            let flag_request = request_builder
                .build_flag_request(&trustee_did, FLAG_NAME.to_string(), FLAG_VALUE.to_string())
                .unwrap();

            let expected_result = json!({
                "type": constants::FLAG,
                "name": FLAG_NAME,
                "value": FLAG_VALUE,
            });

            helpers::check_request_operation(&flag_request, expected_result);
        }
    }

    mod get_flag {
        use super::*;

        #[rstest]
        fn test_pool_build_get_flag(request_builder: RequestBuilder) {
            let attrib_request = request_builder
                .build_get_flag_request(None, FLAG_NAME.to_string(), None, None)
                .unwrap();

            let expected_result = json!({
                "type": constants::GET_FLAG,
                "name": FLAG_NAME,
            });

            helpers::check_request_operation(&attrib_request, expected_result);
        }
    }
}

#[cfg(test)]
#[cfg(feature = "local_nodes_pool")]
mod send_flag {
    use super::*;
    use crate::utils::crypto::Identity;
    use crate::utils::pool::TestPool;

    #[rstest]
    fn test_pool_send_flag(pool: TestPool, trustee: Identity) {
        // Create Flag Request
        let mut flag_request = pool
            .request_builder()
            .build_flag_request(&trustee.did, FLAG_NAME.to_string(), FLAG_VALUE.to_string())
            .unwrap();
        // Send Flag Request
        let flag_response =
            helpers::sign_and_send_request(&trustee, &pool, &mut flag_request).unwrap();

        // Create get_flag request -> This should return the FLAG_VALUE written previously
        let get_flag_request = pool
            .request_builder()
            .build_get_flag_request(None, FLAG_NAME.to_string(), None, None)
            .unwrap();
        let response = pool
            .send_request_with_retries(&get_flag_request, &flag_response)
            .unwrap();
        let data = helpers::get_response_data(&response).unwrap();
        assert_eq!(data["value"].as_str().unwrap(), FLAG_VALUE.to_string());

        // Crate historic get_flag request -> this should error (transaction did not exist at that pont in time)
        let timestamp = data["lut"].as_u64().unwrap();
        let get_flag_request = pool
            .request_builder()
            .build_get_flag_request(None, FLAG_NAME.to_string(), None, Some(timestamp - 1))
            .unwrap();
        let response = pool
            .send_request_with_retries(&get_flag_request, &flag_response)
            .unwrap();
        let data = helpers::get_response_data(&response);
        assert!(data.is_err())
    }
}
