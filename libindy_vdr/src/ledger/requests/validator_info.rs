use super::constants::GET_VALIDATOR_INFO;
use super::RequestType;

#[derive(Serialize, PartialEq, Debug)]
pub struct GetValidatorInfoOperation {
    #[serde(rename = "type")]
    pub _type: String,
}

impl GetValidatorInfoOperation {
    pub fn new() -> GetValidatorInfoOperation {
        GetValidatorInfoOperation {
            _type: Self::get_txn_type().to_string(),
        }
    }
}

impl RequestType for GetValidatorInfoOperation {
    fn get_txn_type<'a>() -> &'a str {
        GET_VALIDATOR_INFO
    }
}
