#[macro_use]
mod utils;

inject_dependencies!();

use indy_vdr::common::did::DidValue;
use indy_vdr::ledger::constants;

use crate::utils::fixtures::*;

#[test]
fn empty() {
    // Empty test to run module
}

const TEXT: &'static str = "indy agreement";
const VERSION: &'static str = "1.0.0";
const DIGEST: &'static str = "83d907821df1c87db829e96569a11f6fc2e7880acba5e43d07ab786959e13bd3";
const RATIFICATION_TS: u64 = 12345;

#[cfg(test)]
mod builder {
    use super::*;
    use indy_vdr::ledger::RequestBuilder;

    mod txn_author_agreement {
        use super::*;
        use crate::utils::helpers::check_request_operation;

        #[rstest]
        fn test_txn_author_agreement_request(request_builder: RequestBuilder,
                                             trustee_did: DidValue,) {
            let request =
                request_builder
                    .build_txn_author_agreement_request(&trustee_did,
                                                        Some(TEXT.to_string()),
                                                        VERSION.to_string(),
                                                        Some(RATIFICATION_TS),
                                                        None).unwrap();

            let expected_operation = json!({
                "type": constants::TXN_AUTHR_AGRMT,
                "text": TEXT,
                "version": VERSION,
                "ratification_ts": RATIFICATION_TS
            });

            check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_txn_author_agreement_request_for_update_ratification_ts(request_builder: RequestBuilder,
                                                                        trustee_did: DidValue) {
            let request =
                request_builder
                    .build_txn_author_agreement_request(&trustee_did,
                                                        None,
                                                        VERSION.to_string(),
                                                        Some(RATIFICATION_TS),
                                                        None).unwrap();

            let expected_operation = json!({
                "type": constants::TXN_AUTHR_AGRMT,
                "version": VERSION,
                "ratification_ts": RATIFICATION_TS
            });

            check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_txn_author_agreement_request_for_make_taa_retired(request_builder: RequestBuilder,
                                                                  trustee_did: DidValue) {
            let request =
                request_builder
                    .build_txn_author_agreement_request(&trustee_did,
                                                        None,
                                                        VERSION.to_string(),
                                                        None,
                                                        Some(54321)).unwrap();

            let expected_operation = json!({
                "type": constants::TXN_AUTHR_AGRMT,
                "version": VERSION,
                "retirement_ts": 54321,
            });

            check_request_operation(&request, expected_operation);
        }
    }

    mod get_txn_author_agreement {
        use super::*;
        use crate::utils::helpers::check_request_operation;
        use indy_vdr::ledger::requests::author_agreement::GetTxnAuthorAgreementData;

        #[rstest]
        fn test_get_txn_author_agreement_request(request_builder: RequestBuilder) {
            let request =
                request_builder
                    .build_get_txn_author_agreement_request(None, None).unwrap();

            let expected_operation = json!({
                "type": constants::GET_TXN_AUTHR_AGRMT,
            });

            check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_get_txn_author_agreement_request_for_digest(request_builder: RequestBuilder) {
            let data = GetTxnAuthorAgreementData {
                digest: Some(DIGEST.to_string()),
                version: None,
                timestamp: None
            };

            let request =
                request_builder
                    .build_get_txn_author_agreement_request(None, Some(&data)).unwrap();

            let expected_operation = json!({
                "type": constants::GET_TXN_AUTHR_AGRMT,
                "digest": DIGEST,
            });

            check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_get_txn_author_agreement_request_for_version(request_builder: RequestBuilder) {
            let data = GetTxnAuthorAgreementData {
                digest: None,
                version: Some(VERSION.to_string()),
                timestamp: None
            };

            let request =
                request_builder
                    .build_get_txn_author_agreement_request(None, Some(&data)).unwrap();

            let expected_operation = json!({
                "type": constants::GET_TXN_AUTHR_AGRMT,
                "version": VERSION,
            });

            check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_get_txn_author_agreement_request_for_timestamp(request_builder: RequestBuilder) {
            let timestamp = 123456789;

            let data = GetTxnAuthorAgreementData {
                digest: None,
                version: None,
                timestamp: Some(timestamp)
            };

            let request =
                request_builder
                    .build_get_txn_author_agreement_request(None, Some(&data)).unwrap();

            let expected_operation = json!({
                "type": constants::GET_TXN_AUTHR_AGRMT,
                "timestamp": timestamp,
            });

            check_request_operation(&request, expected_operation);
        }
    }
}