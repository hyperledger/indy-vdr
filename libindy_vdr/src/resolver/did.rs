use std::collections::HashMap;

use once_cell::sync::Lazy;
use percent_encoding::percent_decode;
use regex::Regex;
use url::Url;

use crate::common::error::prelude::*;
use crate::utils::did::DidValue;

// Patterns to build regular expressions for ledger objects
static DID_INDY_PREFIX: &str = "did:indy";
static NAMESPACE_PATTERN: &str = "((?:[a-z0-9_-]+:?){1,2})";
// uses base58 alphabet
static INDY_UNQUALIFIED_DID_PATTERN: &str =
    "([123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz]{21,22})";
static OBJECT_FAMILY_PATTERN: &str = "([a-z]*)";
static OBJECT_FAMILY_VERSION_PATTERN: &str = "([a-zA-Z0-9]*)";

static ANONCREDSV0_OBJECTS_PATTERN: &str =
    "(SCHEMA|CLAIM_DEF|REV_REG_DEF|REV_REG_ENTRY|REV_REG_DELTA)";

static CLIENT_DEFINED_NAME_PATTERN: &str = "([\\w -]*)";
static SEQ_NO_PATTERN: &str = "(\\d*)";
static VERSION_PATTERN: &str = "((\\d*\\.){1,2}\\d*)";

static DID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        format!(
            r"{}:{}:{}([^\?]+)?(?:\?(.+))?$",
            DID_INDY_PREFIX, NAMESPACE_PATTERN, INDY_UNQUALIFIED_DID_PATTERN
        )
        .as_str(),
    )
    .unwrap()
});

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum QueryParameter {
    VersionId,
    VersionTime,
    From,
    To,
}

impl QueryParameter {
    pub fn parse(input: &str) -> VdrResult<QueryParameter> {
        match input {
            "versionId" => Ok(QueryParameter::VersionId),
            "versionTime" => Ok(QueryParameter::VersionTime),
            "from" => Ok(QueryParameter::From),
            "to" => Ok(QueryParameter::To),
            _ => Err(err_msg(
                VdrErrorKind::Resolver,
                format!("Query parameter {} not supported", input),
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ObjectFamily {
    Anoncreds,
}

impl ObjectFamily {
    fn parse(input: &str) -> VdrResult<ObjectFamily> {
        match input {
            "anoncreds" => Ok(ObjectFamily::Anoncreds),
            _ => Err(err_msg(
                VdrErrorKind::Resolver,
                format!("Object family {} not supported", input),
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Anoncreds {
    AnoncredsV0,
}

impl Anoncreds {
    fn parse(input: &str) -> VdrResult<Anoncreds> {
        match input {
            "v0" => Ok(Anoncreds::AnoncredsV0),
            _ => Err(err_msg(
                VdrErrorKind::Resolver,
                format!("Anoncreds version {} not supported", input),
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Schema {
    pub name: String,
    pub version: String,
}

impl Schema {
    fn new(name: String, version: String) -> Self {
        Self { name, version }
    }

    fn parse(input: &str) -> VdrResult<Schema> {
        let re =
            Regex::new(format!(r"^{}/{}", CLIENT_DEFINED_NAME_PATTERN, VERSION_PATTERN).as_str())
                .unwrap();

        let captures = re.captures(input);

        match captures {
            Some(cap) => Ok(Schema::new(
                cap.get(1)
                    .ok_or_else(|| {
                        err_msg(
                            VdrErrorKind::Resolver,
                            format!("Invalid DID URL path for schema {}", input),
                        )
                    })?
                    .as_str()
                    .to_string(),
                cap.get(2)
                    .ok_or_else(|| {
                        err_msg(
                            VdrErrorKind::Resolver,
                            format!("Invalid DID URL path for schema {}", input),
                        )
                    })?
                    .as_str()
                    .to_string(),
            )),
            _ => Err(err_msg(
                VdrErrorKind::Resolver,
                format!("Invalid DID URL path for schema {}", input),
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ClaimDef {
    pub schema_seq_no: u32,
    pub name: String,
}

impl ClaimDef {
    fn new(schema_seq_no: u32, name: String) -> Self {
        Self {
            schema_seq_no,
            name,
        }
    }

    fn parse(input: &str) -> VdrResult<ClaimDef> {
        let re =
            Regex::new(format!(r"^{}/{}", SEQ_NO_PATTERN, CLIENT_DEFINED_NAME_PATTERN).as_str())
                .unwrap();

        let captures = re.captures(input);

        match captures {
            Some(cap) => Ok(ClaimDef::new(
                cap.get(1)
                    .ok_or_else(|| {
                        err_msg(
                            VdrErrorKind::Resolver,
                            format!("Invalid DID URL path for claim def {}", input),
                        )
                    })?
                    .as_str()
                    .to_string()
                    .parse::<u32>()
                    .unwrap(),
                cap.get(2)
                    .ok_or_else(|| {
                        err_msg(
                            VdrErrorKind::Resolver,
                            format!("Invalid DID URL path for claim def {}", input),
                        )
                    })?
                    .as_str()
                    .to_string(),
            )),
            _ => Err(err_msg(
                VdrErrorKind::Resolver,
                format!("Invalid DID URL path for claim def {}", input),
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct RevReg {
    pub schema_seq_no: u32,
    pub claim_def_name: String,
    pub tag: String,
}

impl RevReg {
    fn new(schema_seq_no: u32, claim_def_name: String, tag: String) -> Self {
        Self {
            schema_seq_no,
            claim_def_name,
            tag,
        }
    }

    fn parse(input: &str) -> VdrResult<RevReg> {
        let re = Regex::new(
            format!(r"^{}/{}/{1}", SEQ_NO_PATTERN, CLIENT_DEFINED_NAME_PATTERN).as_str(),
        )
        .unwrap();

        let captures = re.captures(input);

        match captures {
            Some(cap) => Ok(RevReg::new(
                cap.get(1)
                    .ok_or_else(|| {
                        err_msg(
                            VdrErrorKind::Resolver,
                            format!("Invalid DID URL path for revocation registry {}", input),
                        )
                    })?
                    .as_str()
                    .to_string()
                    .parse::<u32>()
                    .unwrap(),
                cap.get(2)
                    .ok_or_else(|| {
                        err_msg(
                            VdrErrorKind::Resolver,
                            format!("Invalid DID URL path for revocation registry {}", input),
                        )
                    })?
                    .as_str()
                    .to_string(),
                cap.get(3)
                    .ok_or_else(|| {
                        err_msg(
                            VdrErrorKind::Resolver,
                            format!("Invalid DID URL path for revocation registry {}", input),
                        )
                    })?
                    .as_str()
                    .to_string(),
            )),
            _ => Err(err_msg(
                VdrErrorKind::Resolver,
                format!("Invalid DID URL path for revocation registry {}", input),
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LedgerObject {
    Schema(Schema),
    ClaimDef(ClaimDef),
    RevRegDef(RevReg),
    RevRegEntry(RevReg),
    RevRegDelta(RevReg),
}

impl LedgerObject {
    pub fn parse(input: &str) -> VdrResult<LedgerObject> {
        let re = Regex::new(
            format!(
                r"{}/{}/{}/(.+)?",
                OBJECT_FAMILY_PATTERN, OBJECT_FAMILY_VERSION_PATTERN, ANONCREDSV0_OBJECTS_PATTERN
            )
            .as_str(),
        )
        .unwrap();

        let captures = re.captures(input);

        if let Some(cap) = captures {
            let object_family_str = cap
                .get(1)
                .ok_or_else(|| err_msg(VdrErrorKind::Resolver, "Invalid DID URL path"))?
                .as_str();
            let version = cap
                .get(2)
                .ok_or_else(|| err_msg(VdrErrorKind::Resolver, "Invalid DID URL path"))?
                .as_str();

            let object_family = ObjectFamily::parse(object_family_str)?;

            match object_family {
                ObjectFamily::Anoncreds => {
                    let object_family_versioned = Anoncreds::parse(version)?;
                    match object_family_versioned {
                        Anoncreds::AnoncredsV0 => {
                            let ledger_object_type_str = cap
                                .get(3)
                                .ok_or_else(|| {
                                    err_msg(VdrErrorKind::Resolver, "Invalid DID URL path")
                                })?
                                .as_str();
                            let ledger_object_type_specific_str = cap
                                .get(4)
                                .ok_or_else(|| {
                                    err_msg(VdrErrorKind::Resolver, "Invalid DID URL path")
                                })?
                                .as_str();
                            match ledger_object_type_str {
                                "SCHEMA" => Ok(LedgerObject::Schema(Schema::parse(
                                    ledger_object_type_specific_str,
                                )?)),
                                "CLAIM_DEF" => Ok(LedgerObject::ClaimDef(ClaimDef::parse(
                                    ledger_object_type_specific_str,
                                )?)),
                                "REV_REG_DEF" => Ok(LedgerObject::RevRegDef(RevReg::parse(
                                    ledger_object_type_specific_str,
                                )?)),
                                "REV_REG_ENTRY" => Ok(LedgerObject::RevRegEntry(RevReg::parse(
                                    ledger_object_type_specific_str,
                                )?)),
                                "REV_REG_DELTA" => Ok(LedgerObject::RevRegDelta(RevReg::parse(
                                    ledger_object_type_specific_str,
                                )?)),

                                _ => Err(err_msg(
                                    VdrErrorKind::Resolver,
                                    format!(
                                        "Unknown ledger object type {}",
                                        ledger_object_type_str
                                    ),
                                )),
                            }
                        }
                    }
                }
            }
        } else {
            Err(err_msg(
                VdrErrorKind::Resolver,
                format!("Invalid DID URL path for ledger object {}", input),
            ))
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DidUrl {
    pub namespace: String,
    pub id: DidValue,
    pub path: Option<String>,
    pub query: HashMap<QueryParameter, String>,
    pub url: String,
}

impl DidUrl {
    pub fn parse(input: &str) -> VdrResult<DidUrl> {
        let url = Url::parse(input)
            .map_err(|_| err_msg(VdrErrorKind::Resolver, "Could not parse DID Url"))?;
        let mut query_pairs: HashMap<QueryParameter, String> = HashMap::new();
        let _query_pairs: HashMap<_, _> = url.query_pairs().into_owned().collect();

        for (k, v) in _query_pairs.iter() {
            let qp = QueryParameter::parse(k)?;

            query_pairs.insert(qp, v.to_string());
        }

        let captures = DID_REGEX.captures(input.trim());
        match captures {
            Some(cap) => {
                let path = cap
                    .get(3)
                    .map(|p| {
                        percent_decode(p.as_str().as_bytes())
                            .decode_utf8()
                            .map(|p| p.into_owned())
                    })
                    .transpose()
                    .map_err(|_| err_msg(VdrErrorKind::Resolver, "Invalid DID Url path"))?;
                let did = DidUrl {
                    namespace: cap[1].to_string(),
                    id: DidValue::new(&cap[2], Option::None),
                    path,
                    query: query_pairs,
                    url: input.to_string(),
                };
                Ok(did)
            }
            None => Err(err_msg(
                VdrErrorKind::Resolver,
                format!("Invalid DID URL {}", input),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_unknown_ledger_object_fails() {
        let _err = LedgerObject::parse("/anoncreds/v0/PANTS/npdb/4.3.4").unwrap_err();
    }

    #[test]
    fn parse_unknown_object_family_fails() {
        let _err = LedgerObject::parse("/othercreds/v0/SCHEMA/npdb/4.3.4").unwrap_err();
    }

    #[test]
    fn parse_unknown_anoncreds_version_fails() {
        let _err = LedgerObject::parse("/anoncreds/v5/SCHEMA/npdb/4.3.4").unwrap_err();
    }

    #[test]
    fn parse_to_schema() {
        assert_eq!(
            LedgerObject::parse("/anoncreds/v0/SCHEMA/npdb/4.3.4").unwrap(),
            LedgerObject::Schema(Schema::new(String::from("npdb"), String::from("4.3.4")))
        )
    }

    #[test]
    fn parse_to_schema_two_point_seperated_version() {
        assert_eq!(
            LedgerObject::parse("/anoncreds/v0/SCHEMA/npdb/4.3").unwrap(),
            LedgerObject::Schema(Schema::new(String::from("npdb"), String::from("4.3")))
        )
    }

    #[test]
    fn parse_to_schema_two_digit_version() {
        assert_eq!(
            LedgerObject::parse("/anoncreds/v0/SCHEMA/npdb/11.3").unwrap(),
            LedgerObject::Schema(Schema::new(String::from("npdb"), String::from("11.3")))
        )
    }

    #[test]
    fn parse_to_schema_without_version_fails() {
        let _err = LedgerObject::parse("/anoncreds/v0/SCHEMA/npdb").unwrap_err();
    }

    #[test]
    fn parse_to_schema_with_one_digit_version_fails() {
        let _err = LedgerObject::parse("/anoncreds/v0/SCHEMA/npdb/4").unwrap_err();
    }

    #[test]
    fn parse_to_claim_def() {
        assert_eq!(
            LedgerObject::parse("/anoncreds/v0/CLAIM_DEF/23452/npdb").unwrap(),
            LedgerObject::ClaimDef(ClaimDef::new(23452, String::from("npdb")))
        )
    }

    #[test]
    fn parse_to_claim_def_without_seq_no_fails() {
        let _err = LedgerObject::parse("/anoncreds/v0/CLAIM_DEF/npdb").unwrap_err();
    }

    #[test]
    fn parse_to_claim_def_with_seq_no_as_string_fails() {
        let _err = LedgerObject::parse("/anoncreds/v0/CLAIM_DEF/myseqno/npdb").unwrap_err();
    }

    #[test]
    fn parse_to_rev_reg_entry() {
        assert_eq!(
            LedgerObject::parse("/anoncreds/v0/REV_REG_ENTRY/104/revocable/a4e25e54").unwrap(),
            LedgerObject::RevRegEntry(RevReg::new(
                104,
                String::from("revocable"),
                String::from("a4e25e54")
            ))
        )
    }

    #[test]
    fn parse_to_rev_reg_def() {
        assert_eq!(
            LedgerObject::parse(
                "/anoncreds/v0/REV_REG_DEF/104/revocable/a4e25e54-e028-462b-a4d6-b1d1712d51a1"
            )
            .unwrap(),
            LedgerObject::RevRegDef(RevReg::new(
                104,
                String::from("revocable"),
                String::from("a4e25e54-e028-462b-a4d6-b1d1712d51a1")
            ))
        )
    }

    mod did_syntax_tests {

        use super::*;

        #[test]
        fn did_syntax_tests() {
            let _err = DidUrl::parse("did:indy:onlynamespace").unwrap_err();

            assert_eq!(
                DidUrl::parse("did:indy:idunion:BDrEcHc8Tb4Lb2VyQZWEDE").unwrap(),
                DidUrl {
                    namespace: String::from("idunion"),
                    id: DidValue::new("BDrEcHc8Tb4Lb2VyQZWEDE", None),
                    path: None,
                    query: HashMap::new(),
                    url: String::from("did:indy:idunion:BDrEcHc8Tb4Lb2VyQZWEDE"),
                }
            );

            assert_eq!(
                DidUrl::parse("did:indy:sovrin:staging:6cgbu8ZPoWTnR5Rv5JcSMB").unwrap(),
                DidUrl {
                    namespace: String::from("sovrin:staging"),
                    id: DidValue::new("6cgbu8ZPoWTnR5Rv5JcSMB", None),
                    path: None,
                    query: HashMap::new(),
                    url: String::from("did:indy:sovrin:staging:6cgbu8ZPoWTnR5Rv5JcSMB"),
                }
            );

            let _err = DidUrl::parse("did:indy:illegal:third:namespace:1111111111111111111111")
                .unwrap_err();

            let _err = DidUrl::parse("did:indy:test:12345678901234567890").unwrap_err();
            let _err = DidUrl::parse("did:indy:test:12345678901234567890123").unwrap_err();

            // fails because contains characters not in base58
            // 0
            let _err = DidUrl::parse("did:indy:test:0cgbu8ZPoWTnR5Rv5JcSMB").unwrap_err();
            // O
            let _err = DidUrl::parse("did:indy:test:Ocgbu8ZPoWTnR5Rv5JcSMB").unwrap_err();
            // I
            let _err = DidUrl::parse("did:indy:test:Icgbu8ZPoWTnR5Rv5JcSMB").unwrap_err();
            // l
            let _err = DidUrl::parse("did:indy:test:lcgbu8ZPoWTnR5Rv5JcSMB").unwrap_err();
        }

        #[test]
        fn parse_did_url_with_query_parameter() {
            let mut q = HashMap::new();
            q.insert(QueryParameter::VersionId, String::from("1"));

            assert_eq!(
                DidUrl::parse("did:indy:idunion:BDrEcHc8Tb4Lb2VyQZWEDE?versionId=1").unwrap(),
                DidUrl {
                    namespace: String::from("idunion"),
                    id: DidValue::new("BDrEcHc8Tb4Lb2VyQZWEDE", None),
                    path: None,
                    query: q,
                    url: String::from("did:indy:idunion:BDrEcHc8Tb4Lb2VyQZWEDE?versionId=1"),
                }
            );
        }

        #[test]
        fn parse_did_url_fails_with_arbitrary_query_parameter() {
            let _err =
                DidUrl::parse("did:indy:idunion:BDrEcHc8Tb4Lb2VyQZWEDE?hello=world").unwrap_err();
        }

        #[test]
        fn parse_did_url_with_path() {
            assert_eq!(
                DidUrl::parse("did:indy:idunion:Dk1fRRTtNazyMuK2cr64wp/anoncreds/v0/REV_REG_DEF/104/revocable/a4e25e54-e028-462b-a4d6-b1d1712d51a1")
                    .unwrap(),
                DidUrl {
                    namespace: String::from("idunion"),
                    id: DidValue::new("Dk1fRRTtNazyMuK2cr64wp", None),
                    path: Some(String::from("/anoncreds/v0/REV_REG_DEF/104/revocable/a4e25e54-e028-462b-a4d6-b1d1712d51a1")),
                    query: HashMap::new(),
                    url: String::from(
                        "did:indy:idunion:Dk1fRRTtNazyMuK2cr64wp/anoncreds/v0/REV_REG_DEF/104/revocable/a4e25e54-e028-462b-a4d6-b1d1712d51a1"
                    ),
                }
            );
        }

        #[test]
        fn parse_did_url_with_path_and_query() {
            let mut q = HashMap::new();
            q.insert(QueryParameter::VersionTime, String::from("someXmlDateTime"));

            assert_eq!(
                DidUrl::parse("did:indy:idunion:Dk1fRRTtNazyMuK2cr64wp/anoncreds/v0/REV_REG_DEF/104/revocable/a4e25e54-e028-462b-a4d6-b1d1712d51a1?versionTime=someXmlDateTime")
                    .unwrap(),
                DidUrl {
                    namespace: String::from("idunion"),
                    id: DidValue::new("Dk1fRRTtNazyMuK2cr64wp", None),
                    path: Some(String::from("/anoncreds/v0/REV_REG_DEF/104/revocable/a4e25e54-e028-462b-a4d6-b1d1712d51a1")),
                    query: q,
                    url: String::from(
                        "did:indy:idunion:Dk1fRRTtNazyMuK2cr64wp/anoncreds/v0/REV_REG_DEF/104/revocable/a4e25e54-e028-462b-a4d6-b1d1712d51a1?versionTime=someXmlDateTime"
                    ),
                }
            );
        }
    }
}
