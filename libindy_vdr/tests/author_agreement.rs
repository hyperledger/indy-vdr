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

const TEXT: &str = "indy agreement";
const VERSION: &str = "1.0.0";
const DIGEST: &str = "83d907821df1c87db829e96569a11f6fc2e7880acba5e43d07ab786959e13bd3";
const RATIFICATION_TS: u64 = 12345;

#[cfg(test)]
mod builder {
    use super::*;
    use indy_vdr::ledger::RequestBuilder;

    mod txn_author_agreement {
        use super::*;
        use crate::utils::helpers::check_request_operation;

        #[rstest]
        fn test_txn_author_agreement_request(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_txn_author_agreement_request(
                    &trustee_did,
                    Some(TEXT.to_string()),
                    VERSION.to_string(),
                    Some(RATIFICATION_TS),
                    None,
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::TXN_AUTHR_AGRMT,
                "text": TEXT,
                "version": VERSION,
                "ratification_ts": RATIFICATION_TS
            });

            check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_txn_author_agreement_request_for_update_ratification_ts(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_txn_author_agreement_request(
                    &trustee_did,
                    None,
                    VERSION.to_string(),
                    Some(RATIFICATION_TS),
                    None,
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::TXN_AUTHR_AGRMT,
                "version": VERSION,
                "ratification_ts": RATIFICATION_TS
            });

            check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_txn_author_agreement_request_for_make_taa_retired(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_txn_author_agreement_request(
                    &trustee_did,
                    None,
                    VERSION.to_string(),
                    None,
                    Some(54321),
                )
                .unwrap();

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
            let request = request_builder
                .build_get_txn_author_agreement_request(None, None)
                .unwrap();

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
                timestamp: None,
            };

            let request = request_builder
                .build_get_txn_author_agreement_request(None, Some(&data))
                .unwrap();

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
                timestamp: None,
            };

            let request = request_builder
                .build_get_txn_author_agreement_request(None, Some(&data))
                .unwrap();

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
                timestamp: Some(timestamp),
            };

            let request = request_builder
                .build_get_txn_author_agreement_request(None, Some(&data))
                .unwrap();

            let expected_operation = json!({
                "type": constants::GET_TXN_AUTHR_AGRMT,
                "timestamp": timestamp,
            });

            check_request_operation(&request, expected_operation);
        }
    }

    mod disable_all {
        use super::*;
        use crate::utils::helpers::check_request_operation;

        #[rstest]
        fn test_build_disable_all_taa_request(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_disable_all_txn_author_agreements_request(&trustee_did)
                .unwrap();

            let expected_operation = json!({
                "type": constants::DISABLE_ALL_TXN_AUTHR_AGRMTS,
            });

            check_request_operation(&request, expected_operation);
        }
    }

    mod prepare_txn_author_agreement_acceptance_data {
        use super::*;
        use indy_vdr::ledger::requests::author_agreement::TxnAuthrAgrmtAcceptanceData;

        const TEXT: &str = "some agreement text";
        const VERSION: &str = "1.0.0";
        const HASH: &str = "050e52a57837fff904d3d059c8a123e3a04177042bf467db2b2c27abd8045d5e";
        const ACCEPTANCE_MECH_TYPE: &str = "acceptance type 1";
        const TIME_OF_ACCEPTANCE: u64 = 123456789;
        const ROUNDED_TIME_OF_ACCEPTANCE: u64 = 123379200;

        fn _check_acceptance_data(data: TxnAuthrAgrmtAcceptanceData) {
            let expected_data = TxnAuthrAgrmtAcceptanceData {
                mechanism: ACCEPTANCE_MECH_TYPE.to_string(),
                taa_digest: HASH.to_string(),
                time: ROUNDED_TIME_OF_ACCEPTANCE,
            };

            assert_eq!(data, expected_data);
        }

        #[rstest]
        fn test_prepare_txn_author_agreement_acceptance_data_for_text_version(
            request_builder: RequestBuilder,
        ) {
            let data = request_builder
                .prepare_txn_author_agreement_acceptance_data(
                    Some(TEXT),
                    Some(VERSION),
                    None,
                    ACCEPTANCE_MECH_TYPE,
                    TIME_OF_ACCEPTANCE,
                )
                .unwrap();
            _check_acceptance_data(data)
        }

        #[rstest]
        fn test_prepare_txn_author_agreement_acceptance_data_for_hash(
            request_builder: RequestBuilder,
        ) {
            let data = request_builder
                .prepare_txn_author_agreement_acceptance_data(
                    None,
                    None,
                    Some(HASH),
                    ACCEPTANCE_MECH_TYPE,
                    TIME_OF_ACCEPTANCE,
                )
                .unwrap();
            _check_acceptance_data(data)
        }

        #[rstest]
        fn test_prepare_txn_author_agreement_acceptance_data_for_text_version_hash(
            request_builder: RequestBuilder,
        ) {
            let data = request_builder
                .prepare_txn_author_agreement_acceptance_data(
                    Some(TEXT),
                    Some(VERSION),
                    Some(HASH),
                    ACCEPTANCE_MECH_TYPE,
                    TIME_OF_ACCEPTANCE,
                )
                .unwrap();
            _check_acceptance_data(data)
        }

        #[rstest]
        fn test_prepare_txn_author_agreement_acceptance_data_for_text_version_not_correspond_to_hash(
            request_builder: RequestBuilder,
        ) {
            let _err = request_builder
                .prepare_txn_author_agreement_acceptance_data(
                    Some("Other TEXT"),
                    Some(VERSION),
                    Some(HASH),
                    ACCEPTANCE_MECH_TYPE,
                    TIME_OF_ACCEPTANCE,
                )
                .unwrap_err();
        }

        #[rstest]
        fn test_prepare_txn_author_agreement_acceptance_data_for_missed_text_version_hash(
            request_builder: RequestBuilder,
        ) {
            let _err = request_builder
                .prepare_txn_author_agreement_acceptance_data(
                    None,
                    None,
                    None,
                    ACCEPTANCE_MECH_TYPE,
                    TIME_OF_ACCEPTANCE,
                )
                .unwrap_err();
        }
    }
}

#[cfg(test)]
#[cfg(feature = "local_nodes_pool")]
mod author_agreement {
    use super::*;
    use crate::utils::crypto::Identity;
    use crate::utils::helpers;
    use crate::utils::pool::TestPool;
    use indy_vdr::pool::PreparedRequest;

    fn _build_nym_request(pool: &TestPool, trustee: &Identity) -> PreparedRequest {
        let new_identity = Identity::new(None);

        pool.request_builder()
            .build_nym_request(
                &trustee.did,
                &new_identity.did,
                Some(new_identity.verkey.to_string()),
                None,
                None,
            )
            .unwrap()
    }

    fn _accept_taa(
        request: &mut PreparedRequest,
        pool: &TestPool,
        taa_text: &str,
        taa_version: &str,
        aml_label: &str,
    ) {
        let data = pool
            .request_builder()
            .prepare_txn_author_agreement_acceptance_data(
                Some(&taa_text),
                Some(&taa_version),
                None,
                &aml_label,
                helpers::current_timestamp(),
            )
            .unwrap();

        request.set_txn_author_agreement_acceptance(&data).unwrap();
    }

    #[rstest]
    fn test_pool_transaction_author_agreement(pool: TestPool, trustee: Identity) {
        // Set AML on the Ledger
        let (_response, _aml, aml_label, _aml_version, _aml_context) =
            helpers::taa::set_aml(&pool, &trustee);

        // Set TAA on the Ledger
        let (response, taa_text, taa_version, _ratification_ts) =
            helpers::taa::set_taa(&pool, &trustee);

        // Get Taa from the ledger
        let _get_taa_response = helpers::taa::get_taa(&pool, &response, &taa_version);

        // Try to publish new NYM without accepting TAA
        let new_identity = Identity::new(None);

        let mut nym_request = _build_nym_request(&pool, &trustee);
        let err = helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap_err();
        helpers::check_response_type(&err, "REJECT");

        // Publish new NYM with accepting TAA
        _accept_taa(&mut nym_request, &pool, &taa_text, &taa_version, &aml_label);

        let nym_response =
            helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap();

        // Ensure NYM is written
        let get_nym_request = pool
            .request_builder()
            .build_get_nym_request(None, &new_identity.did)
            .unwrap();

        let _response = pool
            .send_request_with_retries(&get_nym_request, &nym_response)
            .unwrap();

        // Disable TAA
        helpers::taa::disable_taa(&pool, &trustee);

        // Send new NYM without accepting TAA
        let mut nym_request = _build_nym_request(&pool, &trustee);
        let _response = helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap();

        // Try to send transactions using invalid TAA
        _author_agreement_for_using_invalid_taa(&pool, &trustee, &aml_label);

        // Multiple TAA
        _author_agreement_for_retire_taa(&pool, &trustee, &aml_label);

        // TAA Ratification data
        _author_agreement_for_ratification_time(&pool, &trustee, &aml_label);
    }

    fn _author_agreement_for_using_invalid_taa(
        pool: &TestPool,
        trustee: &Identity,
        aml_label: &str,
    ) {
        // Set TAA on the Ledger
        let (_response, taa_text, taa_version, _ratification_ts) =
            helpers::taa::set_taa(&pool, &trustee);

        // Try to publish NYM with accepting different TAA text
        let mut nym_request = _build_nym_request(&pool, &trustee);

        let data = pool
            .request_builder()
            .prepare_txn_author_agreement_acceptance_data(
                Some(&"DIFFERENT TAA TEXT"),
                Some(&taa_version),
                None,
                &aml_label,
                helpers::current_timestamp(),
            )
            .unwrap();

        nym_request
            .set_txn_author_agreement_acceptance(&data)
            .unwrap();

        let err = helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap_err();
        helpers::check_response_type(&err, "REJECT");

        // Try to publish NYM with accepting different TAA acceptance type
        let data = pool
            .request_builder()
            .prepare_txn_author_agreement_acceptance_data(
                Some(&taa_text),
                Some(&taa_version),
                None,
                "DIFFERENT ACCPTANCE TYPE",
                helpers::current_timestamp(),
            )
            .unwrap();

        nym_request
            .set_txn_author_agreement_acceptance(&data)
            .unwrap();

        let err = helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap_err();
        helpers::check_response_type(&err, "REJECT");

        // Disable TAA
        helpers::taa::disable_taa(&pool, &trustee);
    }

    fn _author_agreement_for_retire_taa(pool: &TestPool, trustee: &Identity, aml_label: &str) {
        // Set TAA on the Ledger
        let (txn_author_agreement_response, taa_text, taa_version, ratification_ts) =
            helpers::taa::set_taa(&pool, &trustee);

        let expected_data =
            json!({"text": taa_text, "version": taa_version, "ratification_ts": ratification_ts});
        helpers::taa::check_taa(
            &pool,
            &txn_author_agreement_response,
            &taa_version,
            expected_data,
        );

        // Set TAA 2 on the Ledger to be able make the first one retired
        let (_txn_author_agreement_response_2, taa_text_2, taa_version_2, ratification_ts_2) =
            helpers::taa::set_taa(&pool, &trustee);

        let expected_data = json!({"text": taa_text_2, "version": taa_version_2, "ratification_ts": ratification_ts_2});
        helpers::taa::check_taa(
            &pool,
            &txn_author_agreement_response,
            &taa_version_2,
            expected_data,
        );

        // Now both TAA are valid
        // Send NYM using TAA 1
        let mut nym_request = _build_nym_request(&pool, &trustee);
        _accept_taa(&mut nym_request, &pool, &taa_text, &taa_version, &aml_label);
        let _response = helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap();

        // Send NYM using TAA 2
        let mut nym_request_2 = _build_nym_request(&pool, &trustee);
        _accept_taa(
            &mut nym_request_2,
            &pool,
            &taa_text_2,
            &taa_version_2,
            &aml_label,
        );
        let _response =
            helpers::sign_and_send_request(&trustee, &pool, &mut nym_request_2).unwrap();

        // Update first TAA to make retired
        let retirement_ts = helpers::current_timestamp() - 60 * 60 * 24;

        let mut request = pool
            .request_builder()
            .build_txn_author_agreement_request(
                &trustee.did,
                None,
                taa_version.to_string(),
                Some(ratification_ts),
                Some(retirement_ts),
            )
            .unwrap();

        helpers::sign_and_send_request(&trustee, &pool, &mut request).unwrap();

        // Send NYM using TAA 1 - fail
        let mut nym_request = _build_nym_request(&pool, &trustee);
        _accept_taa(&mut nym_request, &pool, &taa_text, &taa_version, &aml_label);
        let err = helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap_err();
        helpers::check_response_type(&err, "REJECT");

        // Send NYM using TAA 2 - success
        let mut nym_request_2 = _build_nym_request(&pool, &trustee);
        _accept_taa(
            &mut nym_request_2,
            &pool,
            &taa_text_2,
            &taa_version_2,
            &aml_label,
        );
        let _response =
            helpers::sign_and_send_request(&trustee, &pool, &mut nym_request_2).unwrap();

        // Try to make second TAA retired - fail. Cannot make the only TAA retired. Disable ALL myst be used
        let retirement_ts = helpers::current_timestamp() - 60 * 60 * 24;

        let mut request = pool
            .request_builder()
            .build_txn_author_agreement_request(
                &trustee.did,
                None,
                taa_version_2.to_string(),
                Some(ratification_ts),
                Some(retirement_ts),
            )
            .unwrap();

        let err = helpers::sign_and_send_request(&trustee, &pool, &mut request).unwrap_err();
        helpers::check_response_type(&err, "REJECT");

        // Disable all TAA
        helpers::taa::disable_taa(&pool, &trustee);
    }

    fn _author_agreement_for_ratification_time(
        pool: &TestPool,
        trustee: &Identity,
        aml_label: &str,
    ) {
        // Set TAA on the Ledger
        let (_txn_author_agreement_response, taa_text, taa_version, ratification_ts) =
            helpers::taa::set_taa(&pool, &trustee);

        // Send NYM with using acceptance time that earlier ratification_ts
        let mut nym_request = _build_nym_request(&pool, &trustee);

        let data = pool
            .request_builder()
            .prepare_txn_author_agreement_acceptance_data(
                Some(&taa_text),
                Some(&taa_version),
                None,
                &aml_label,
                ratification_ts - 60 * 60 * 24,
            )
            .unwrap();

        nym_request
            .set_txn_author_agreement_acceptance(&data)
            .unwrap();

        let err = helpers::sign_and_send_request(&trustee, &pool, &mut nym_request).unwrap_err();
        helpers::check_response_type(&err, "REJECT");

        // Disable all TAA
        helpers::taa::disable_taa(&pool, &trustee);

        // Setup TAA without ratification timestamp
        let (taa_text, taa_version, _) = helpers::taa::gen_taa_data();

        let mut request = pool
            .request_builder()
            .build_txn_author_agreement_request(
                &trustee.did,
                Some(taa_text.to_string()),
                taa_version.to_string(),
                None,
                None,
            )
            .unwrap();

        let err = helpers::sign_and_send_request(&trustee, &pool, &mut request).unwrap_err();
        helpers::check_response_type(&err, "REJECT");
    }
}
