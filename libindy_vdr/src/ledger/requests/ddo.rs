use super::constants::GET_DDO;
use super::did::ShortDidValue;
use super::RequestType;

#[derive(Serialize, PartialEq, Debug)]
pub struct GetDdoOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub dest: ShortDidValue,
}

impl GetDdoOperation {
    pub fn new(dest: ShortDidValue) -> GetDdoOperation {
        GetDdoOperation {
            _type: Self::get_txn_type().to_string(),
            dest,
        }
    }
}

impl RequestType for GetDdoOperation {
    fn get_txn_type<'a>() -> &'a str {
        GET_DDO
    }
}
