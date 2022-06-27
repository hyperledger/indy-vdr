#[macro_use]
mod utils;

inject_dependencies!();

use indy_vdr::ledger::constants;
use indy_vdr::ledger::requests::author_agreement::AcceptanceMechanisms;
use indy_vdr::utils::did::DidValue;

use crate::utils::fixtures::*;

#[test]
fn empty() {
    // Empty test to run module
}

const VERSION: &str = "1.0.0";
const CONTEXT: &str = "Some aml context";
const TIMESTAMP: u64 = 12345;

fn aml() -> AcceptanceMechanisms {
    let mut aml: AcceptanceMechanisms = AcceptanceMechanisms::new();
    aml.0.insert(
        String::from("acceptance mechanism label 1"),
        json!("some acceptance mechanism description 1"),
    );
    aml.0.insert(
        String::from("acceptance mechanism label 1"),
        json!({"filed": "value"}),
    );
    aml
}

#[cfg(test)]
mod builder {
    use super::*;
    use crate::utils::helpers::check_request_operation;
    use indy_vdr::ledger::RequestBuilder;

    mod author_agreement {
        use super::*;

        #[rstest]
        fn test_build_acceptance_mechanisms_request(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_acceptance_mechanisms_request(&trustee_did, aml(), VERSION.to_string(), None)
                .unwrap();

            let expected_operation = json!({
                "type": constants::TXN_AUTHR_AGRMT_AML,
                "aml": aml(),
                "version": VERSION
            });

            check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_acceptance_mechanisms_request_with_context(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_acceptance_mechanisms_request(
                    &trustee_did,
                    aml(),
                    VERSION.to_string(),
                    Some(CONTEXT.to_string()),
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::TXN_AUTHR_AGRMT_AML,
                "aml": aml(),
                "version": VERSION,
                "amlContext": CONTEXT,
            });

            check_request_operation(&request, expected_operation);
        }
    }

    mod get_txn_author_agreement {
        use super::*;

        #[rstest]
        fn test_get_build_acceptance_mechanisms_request(request_builder: RequestBuilder) {
            let request = request_builder
                .build_get_acceptance_mechanisms_request(None, None, None)
                .unwrap();
            let expected_operation = json!({
                "type": constants::GET_TXN_AUTHR_AGRMT_AML,
            });

            check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_get_build_acceptance_mechanisms_request_for_timestamp(
            request_builder: RequestBuilder,
        ) {
            let request = request_builder
                .build_get_acceptance_mechanisms_request(None, Some(TIMESTAMP), None)
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_TXN_AUTHR_AGRMT_AML,
                "timestamp": TIMESTAMP,
            });

            check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_get_build_acceptance_mechanisms_request_for_version(
            request_builder: RequestBuilder,
        ) {
            let request = request_builder
                .build_get_acceptance_mechanisms_request(None, None, Some(VERSION.to_string()))
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_TXN_AUTHR_AGRMT_AML,
                "version": VERSION,
            });

            check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_get_build_acceptance_mechanisms_request_for_timestamp_and_version(
            request_builder: RequestBuilder,
        ) {
            let _err = request_builder
                .build_get_acceptance_mechanisms_request(
                    None,
                    Some(TIMESTAMP),
                    Some(VERSION.to_string()),
                )
                .unwrap_err();
        }
    }
}

#[cfg(test)]
#[cfg(feature = "local_nodes_pool")]
mod send_aml {
    use super::*;
    use crate::utils::crypto::Identity;
    use crate::utils::helpers;
    use crate::utils::pool::TestPool;

    #[rstest]
    fn test_pool_send_acceptance_mechanisms(pool: TestPool, trustee: Identity) {
        // Set AML on the Ledger
        let (response, aml, _aml_label, aml_version, aml_context) =
            helpers::taa::set_aml(&pool, &trustee);

        // Get AML set on the Ledger
        let get_acceptance_mechanisms_request = pool
            .request_builder()
            .build_get_acceptance_mechanisms_request(None, None, None)
            .unwrap();

        let response = pool
            .send_request_with_retries(&get_acceptance_mechanisms_request, &response)
            .unwrap();

        let response: serde_json::Value = serde_json::from_str(&response).unwrap();
        let expected_data = json!({"aml": aml, "version": aml_version, "amlContext": aml_context});
        assert_eq!(response["result"]["data"], expected_data);
    }
}
