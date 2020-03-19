use crate::ledger::PreparedRequest;
use crate::tests::utils::crypto::Identity;
use crate::tests::utils::pool::TestPool;

pub fn check_request_operation(request: &PreparedRequest, expected_operation: serde_json::Value) {
    assert_eq!(request.req_json["operation"], expected_operation);
}

pub fn check_response_type(response: &str, expected_type: &str) {
    let response_: serde_json::Value = serde_json::from_str(response).unwrap();
    assert_eq!(response_["op"].as_str().unwrap(), expected_type);
}

pub fn get_response_data(response: &str) -> Result<serde_json::Value, String> {
    let response_: serde_json::Value = serde_json::from_str(response).unwrap();
    if !response_["result"]["data"].is_null() {
        return Ok(response_["result"]["data"].to_owned());
    }
    if !response_["result"]["txn"]["data"].is_null() {
        return Ok(response_["result"]["txn"]["data"].to_owned());
    }
    Err(String::from("Cannot get response data"))
}

pub fn new_ledger_identity(pool: &TestPool, role: Option<String>) -> Identity {
    let trustee = Identity::trustee();
    let new_identity = Identity::new(None);

    // Send NYM
    let mut nym_request =
        pool.request_builder()
            .build_nym_request(&trustee.did,
                               &new_identity.did,
                               Some(new_identity.verkey.to_string()),
                               None,
                               role).unwrap();

    trustee.sign_request(&mut nym_request);

    pool.send_request(&nym_request).unwrap();

    new_identity
}