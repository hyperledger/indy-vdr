use std::collections::HashMap;

use indy_vdr::ledger::constants::{
    RICH_SCHEMA, RICH_SCHEMA_CRED_DEF, RICH_SCHEMA_CTX, RICH_SCHEMA_ENCODING, RICH_SCHEMA_MAPPING,
    RICH_SCHEMA_PRES_DEF, RS_CONTEXT_TYPE_VALUE, RS_CRED_DEF_TYPE_VALUE, RS_ENCODING_TYPE_VALUE,
    RS_MAPPING_TYPE_VALUE, RS_PRES_DEF_TYPE_VALUE, RS_SCHEMA_TYPE_VALUE,
};
use once_cell::sync::Lazy;

pub const TRUSTEE_SEED: [u8; 32] = [
    48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
    84, 114, 117, 115, 116, 101, 101, 49,
];
pub const TRUSTEE_DID: &str = "V4SGRU86Z58d6TV7PBUe6f";
pub const TRUSTEE_DID_FQ: &str = "did:sov:V4SGRU86Z58d6TV7PBUe6f";
//pub const TRUSTEE_VERKEY: &str = "GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL";
pub const STEWARD_SEED: [u8; 32] = [
    48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
    83, 116, 101, 119, 97, 114, 100, 49,
];
pub const STEWARD_DID: &str = "V4SGRU86Z58d6TV7PBUe6f"; // TODO: change

// pub const MY1_SEED: [u8; 32] = [
//     48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
//     48, 48, 48, 48, 48, 77, 121, 49,
// ];
pub const MY1_DID: &str = "VsKV7grR1BUE29mG2Fm2kX";
pub const MY1_DID_FQ: &str = "did:sov:VsKV7grR1BUE29mG2Fm2kX";
pub const MY1_VERKEY: &str = "GjZWsBLgZCR18aL468JAT7w9CZRiBnpxUPPgyQxh4voa";

pub static RS_TYPE_TO_OP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    [
        (RS_SCHEMA_TYPE_VALUE, RICH_SCHEMA),
        (RS_ENCODING_TYPE_VALUE, RICH_SCHEMA_ENCODING),
        (RS_CONTEXT_TYPE_VALUE, RICH_SCHEMA_CTX),
        (RS_MAPPING_TYPE_VALUE, RICH_SCHEMA_MAPPING),
        (RS_CRED_DEF_TYPE_VALUE, RICH_SCHEMA_CRED_DEF),
        (RS_PRES_DEF_TYPE_VALUE, RICH_SCHEMA_PRES_DEF),
    ]
    .iter()
    .copied()
    .collect()
});
