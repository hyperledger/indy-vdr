use super::constants::{RICH_SCHEMA};
use super::{RequestType};
use crate::utils::validation::{Validatable, ValidationError};
use crate::ledger::identifiers::rich_schema::{RichSchemaId};

pub const MAX_ATTRIBUTES_COUNT: usize = 125;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RSContent {
    jsld_string : String,
}


impl RSContent {
    // ToDo: Should it be json-ld validated object or something like that? For now, String object using is enough
    pub fn new(jsld_string: String) -> Self {
        Self {
            jsld_string
        }
    }

    pub fn loads(jsld: String) -> Self {
        // ToDo: Add JSON-LD object creation from string
        Self {
            jsld_string: jsld,
        }
    }
}


impl Validatable for RSContent {
    fn validate(&self) -> Result<(), ValidationError> {
        // ToDo: Add JSON-LD validation if needed
        return Ok(())
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RichSchema {
    pub id: RichSchemaId,
    pub content: RSContent,
    pub rs_name: String,
    pub rs_version: String,
    pub rs_type: i32,
    pub ver: i32,
}


impl RichSchema {
    pub fn new(id: RichSchemaId, content: RSContent, rs_name: String, rs_version: String, rs_type: i32, ver: i32) -> Self {
        Self {
            id,
            content,
            rs_name,
            rs_version,
            rs_type,
            ver
        }
    }
}


impl Validatable for RichSchema {
    fn validate(&self) -> Result<(), ValidationError> {
        // ToDo: add specific validation
        return self.id.validate()
        //     Ok(()) => {
        //         match self.content.validate(){
        //             Ok(()) => return Ok(()),
        //             Err(_) => return Err(_),
        //         };
        //     },
        //     Err(_) => return Err(_),
        // }
    }
}


#[derive(Serialize, Debug)]
pub struct RichSchemaOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub id: RichSchemaId,
    pub content: RSContent,
    pub rs_name: String,
    pub rs_version: String,
    pub rs_type: i32,
    pub ver: i32,
}


impl RichSchemaOperation {
    pub fn new(rs_schema: RichSchema) -> RichSchemaOperation {
        RichSchemaOperation {
            _type: Self::get_txn_type().to_string(),
            id: rs_schema.id,
            content: rs_schema.content,
            rs_name: rs_schema.rs_name,
            rs_version: rs_schema.rs_version,
            rs_type: rs_schema.rs_type,
            ver: rs_schema.ver,
        }
    }
}

impl RequestType for RichSchemaOperation {
    fn get_txn_type<'a>() -> &'a str {
        RICH_SCHEMA
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn _rs_schema_v1() -> RichSchema {
        RichSchema::new(
            RichSchemaId::new("did:sov:some_hash_value".to_string()),
            RSContent::new(r#"{"json": "ld"; "valid": "object"}"#.to_string()),
            "test_rich_schema".to_string(),
            "first_version".to_string(),
            42,
            1
        )
    }

    fn _rs_operation() -> RichSchemaOperation {
        RichSchemaOperation::new(_rs_schema_v1())
    }

    #[test]
    fn _check_type() {
        assert_eq!(_rs_operation()._type, RICH_SCHEMA)
    }
}