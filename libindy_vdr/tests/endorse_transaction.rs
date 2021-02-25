#[macro_use]
mod utils;

inject_dependencies!();

#[test]
fn empty() {
    // Empty test to run module
}

#[cfg(test)]
#[cfg(feature = "local_nodes_pool")]
mod endorse_transaction {
    #[allow(unused_imports)]
    use super::*;
    use crate::utils::fixtures::*;
    use crate::utils::helpers;
    use crate::utils::pool::TestPool;

    #[rstest]
    fn test_pool_send_request_by_endorser(pool: TestPool) {
        let endorser = helpers::new_ledger_identity(&pool, Some(String::from("ENDORSER")));

        // Endorse Schema.  Multi sign + Multi Sign.
        let author = helpers::new_ledger_identity(&pool, None);
        let (schema_id, mut schema_request) = helpers::schema::build_schema_request(&pool, &author);
        schema_request.set_endorser(&endorser.did).unwrap();
        author.multi_sign_request(&mut schema_request);
        endorser.multi_sign_request(&mut schema_request);

        let schema_response = pool.send_request(&schema_request).unwrap();

        // Ensure Schema is written correctly
        helpers::schema::ensure_schema_is_written(&pool, &schema_response, &schema_id);

        // Endorse Schema.  Sign + Multi Sign.
        let author = helpers::new_ledger_identity(&pool, None);
        let (schema_id, mut schema_request) = helpers::schema::build_schema_request(&pool, &author);
        schema_request.set_endorser(&endorser.did).unwrap();
        author.sign_request(&mut schema_request);
        endorser.multi_sign_request(&mut schema_request);
        let schema_response = pool.send_request(&schema_request).unwrap();

        // Ensure Schema is written correctly
        helpers::schema::ensure_schema_is_written(&pool, &schema_response, &schema_id);
    }

    #[rstest]
    fn test_pool_send_request_by_endorser_for_missed_one_of_signatures(pool: TestPool) {
        let author = helpers::new_ledger_identity(&pool, None);
        let endorser = helpers::new_ledger_identity(&pool, Some(String::from("ENDORSER")));

        // Endorse Schema signed by author only

        let (_schema_id, mut schema_request) =
            helpers::schema::build_schema_request(&pool, &author);
        schema_request.set_endorser(&endorser.did).unwrap();
        author.multi_sign_request(&mut schema_request);
        let err = pool.send_request(&schema_request).unwrap_err();
        helpers::check_response_type(&err, "REQNACK");

        // Endorse Schema signed by endorser only
        let (_schema_id, mut schema_request) =
            helpers::schema::build_schema_request(&pool, &author);
        schema_request.set_endorser(&endorser.did).unwrap();
        endorser.multi_sign_request(&mut schema_request);
        let err = pool.send_request(&schema_request).unwrap_err();
        helpers::check_response_type(&err, "REQNACK");
    }
}
