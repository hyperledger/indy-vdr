use crate::utils::crypto::Identity;
use crate::utils::pool::TestPool;
use indy_vdr::common::error::VdrResult;
use indy_vdr::pool::{NodeReplies, PreparedRequest};
use indy_vdr::utils::did::DidValue;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde_json::json;

pub fn current_timestamp() -> u64 {
    time::OffsetDateTime::now_utc().unix_timestamp() as u64
}

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
    let mut nym_request = pool
        .request_builder()
        .build_nym_request(
            &trustee.did,
            &new_identity.did,
            Some(new_identity.verkey.to_string()),
            None,
            role,
        )
        .unwrap();

    sign_and_send_request(&trustee, pool, &mut nym_request).unwrap();

    new_identity
}

pub fn sign_and_send_request(
    identity: &Identity,
    pool: &TestPool,
    request: &mut PreparedRequest,
) -> Result<String, String> {
    identity.sign_request(&mut *request);
    pool.send_request(request)
}

pub fn sign_and_send_full_request(
    pool: &TestPool,
    trustee: &Identity,
    node_aliases: Option<Vec<String>>,
    timeout: Option<i64>,
) -> VdrResult<NodeReplies<String>> {
    let mut request = pool
        .request_builder()
        .build_get_validator_info_request(&trustee.did)
        .unwrap();

    trustee.sign_request(&mut request);

    pool.send_full_request(&request, node_aliases, timeout)
}

pub fn rand_string(len: usize) -> String {
    String::from_utf8(
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .collect(),
    )
    .unwrap()
}

fn rand_version() -> String {
    let version: u32 = rand::thread_rng().gen();
    version.to_string()
}

pub mod schema {
    use super::*;
    use indy_vdr::ledger::identifiers::SchemaId;
    use indy_vdr::ledger::requests::schema::{AttributeNames, Schema, SchemaV1};
    use std::collections::HashSet;

    pub const NAME: &str = "gvt";
    pub const VERSION: &str = "1.0";

    pub fn attributes() -> AttributeNames {
        let mut attributes = HashSet::new();
        attributes.insert(String::from("name"));
        AttributeNames(attributes)
    }

    pub fn default_schema(did: &DidValue) -> SchemaV1 {
        SchemaV1 {
            id: build_schema_id(did, NAME, VERSION),
            name: NAME.to_string(),
            version: VERSION.to_string(),
            attr_names: attributes(),
            seq_no: None,
        }
    }

    pub fn new_schema(did: &DidValue) -> SchemaV1 {
        SchemaV1 {
            id: build_schema_id(did, NAME, VERSION),
            name: rand_string(30),
            version: format!("{}.{}", rand_version(), rand_version()),
            attr_names: attributes(),
            seq_no: None,
        }
    }

    pub fn build_schema_id(did: &DidValue, name: &str, version: &str) -> SchemaId {
        SchemaId(format!("{}:2:{}:{}", did.0, name, version))
    }

    pub fn build_schema_request(
        pool: &TestPool,
        identity: &Identity,
    ) -> (SchemaId, PreparedRequest) {
        let schema = schema::new_schema(&identity.did);
        let schema_request = pool
            .request_builder()
            .build_schema_request(&identity.did, Schema::SchemaV1(schema.clone()))
            .unwrap();
        (schema.id, schema_request)
    }

    pub fn publish(identity: &Identity, pool: &TestPool, schema: &SchemaV1) -> (SchemaId, u64) {
        // Send Schema
        let mut schema_request = pool
            .request_builder()
            .build_schema_request(&identity.did, Schema::SchemaV1(schema.clone()))
            .unwrap();

        let schema_response = sign_and_send_request(identity, pool, &mut schema_request).unwrap();

        let seq_no = TestPool::extract_seq_no_from_reply(&schema_response).unwrap();
        (schema.id.clone(), seq_no)
    }

    pub fn ensure_schema_is_written(pool: &TestPool, schema_response: &str, schema_id: &SchemaId) {
        // Get Schema
        let get_schema_request = pool
            .request_builder()
            .build_get_schema_request(None, schema_id)
            .unwrap();

        pool.send_request_with_retries(&get_schema_request, schema_response)
            .unwrap();
    }
}

pub mod cred_def {
    use super::*;
    use indy_vdr::ledger::identifiers::{CredentialDefinitionId, SchemaId};
    use indy_vdr::ledger::requests::cred_def::{
        CredentialDefinition, CredentialDefinitionV1, SignatureType,
    };

    pub const SCHEMA_SEQ_NO: u64 = 1;
    pub const TAG: &str = "tag";
    pub const TYPE: SignatureType = SignatureType::CL;

    pub fn build_cred_def_id(did: &DidValue, schema_seq_no: u64) -> CredentialDefinitionId {
        CredentialDefinitionId(format!(
            "{}:3:{}:{}:{}",
            did.0,
            TYPE.to_str(),
            schema_seq_no,
            TAG
        ))
    }

    pub fn cred_def_data() -> serde_json::Value {
        json!({
            "primary":{
               "n":"120355922221985384840941239923100098467249121192168997520967253987379565887884946081004971905241773186744825621339367540106415318979200429595820993264419931750516061335038027449057333321438253474049202463386762257469932179958034559456489801358679273055900822319445863979806766149039009260758744892140692088997433543626535908497559984530365443943202102380983574304304742424683688514981269429653487513388367632401946482455129868684917532038579078047872197350466225495932960165430763601866452447800767783854420538868860822833413007565528731365160295737564934145787658279168974902910533331251646320611584015178337188371629",
               "s":"103073651899967542148431164440066264186079006603480026043053990019414043746310202103137151279357982313889939249918072626652765223106661122376688930512822613066077256891259342933008294881448032741394615579209856822955715575092189390998315774629117948509361639045529577259967791824850974206010183126350264400555408442879770076027401083832883994483664609758392636235733414252396272045747437401822008966675868211598460789271246646277791808299246790909249745780827685191491554892618231648861626297764714375986367882712617792259382392291711944231451657812143340325007962280275140126719226522630395689568000152728384760761323",
               "r":{
                    "master_secret":"94819462429964782488253960217324081394475931205418718142317526558207681227376929277174093835219025292859246210036551435660923536888489219926088341075790293379476507077251598069902917607541750000580647123989081164604531237874843436317804066263539534802343555570306187128406438610413837780970660191395984787608622094195931619020749529730526730923261578139512746844884030972010612471990000448187332711946037804657190912477854767455602447769012813275307396327702296831386522638664623768996826591855980549460342686684946543842568058957824718300830137931929954093271470868390222205521488050132986916828162584787189097546774",
                    "name":"101387889887306914394935649959396909614249910192459936292070358425760006527433929280373398931907598115501852483637423594947249051903746796725480129861280761775782130635743741170939979477179584626774946295233206326207037538796337735931428820142160333747283353038941896115331692772843574818647345677756701861606098058705032643633731527967774761975632789391466054530479973395667846458984663838845079433716159920055837469455211246382059279230559946054529472907975046850222327286309094430063476868492861304094337038376226823348129284103528231335508365493700772994683276375727799146215389188052657305207786212015609088228721"
                },
               "rctxt":"27212770587089563007412510633602185685469307050447403259728987495138200205683375169591335498466548570390509970629253881837062155701743778647707641991212999975719943682532857227741999616728610834035901407897677508565380402732336307049016971820986746831663957485551357418536095685010723828875101322769670871721498123169639947018636978753925246551229251526159259876997845115999747911572978230468934349483598766964120486325022854698237068014053939595236246691730299239293149782730142666658401574516025290347914742373447372434285745478453453775805709629860474539364959254596395949639165593088684583046696811913210522396082",
               "z":"15423491035243897460139545245296392159617913243841113612037020920738142123240529178950374630507638553310344907832671387893976593044975965545610254862091014447666951151951728311507948295379500974657874782734549983791467070744197079136379645038057151203035190719598316826503656282505557169060279961262321473632125218731700869764577506040315713547186932222945059293898245138047331955319829849258199247450901117393283317687507714395437337925983773404038919162245837544470258849543325439831240965120949600927756230091278095620367777795618295757858533759540906536439723654446781219895899090904751491039040356391538718477883"
            },
            "revocation":{
                "g": "1 1F5E78FABE52493378EDFCBD0932C161DF607E7D38085D2DD3397BB92792AF33 1 01E6A82FD70510D154C90EADE40C62ED935AEEF76F2F873E9CC142E7DF6EE8E9 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8",
                "g_dash": "1 00DA2EE5353EAFB0CD29B8FF674E07CCA50BF40E45FDF2169BBD4F95FB55A996 1 0F6427E722E9BF01BC7273D305991786E06D8FD9A3F058F2936FA776345CAF2E 1 05B48C396AF61751218EA21CF191D71FA0F2110FB60BEA47E7981546FCFFD71D 1 11ADCE38419AB5DAEDFF96998EEB8411DC3416E2911A273EBE3D4F1C55DA146D 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000",
                "h": "1 22D864A9C3669043E437D444404BCCC3D859F8C2D18F894F93F9E1C1D1CA9AA3 1 1C3D2784736CC630341B6C009A5A83E10A34376C829447605258B41D5DBEFF7C 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8",
                "h0": "1 21FC814C76EA10893945B465D4BB71CDA3D1417D8A21059B93E597EC789F2706 1 12C1DDB15288C8617DCF866684ACEBE1AE8695B67C4C8D292761289A409085E1 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8",
                "h1":"1 0CD5A39085CACDE95004101E150E2F2EE80C9FB1F32E58C37DB0B817A4524FB9 1 2055163290C99C13344F63C43D0998259A2FEF0489D8270EF26F467972508A60 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8",
                "h2":"1 09B6532F727B9923BC66D97AAAA0A66C2C035DF3C193B8A929EC0A290727CFD8 1 108F99FC239CB588BFD4436E9B583731D8ED3247A838105B0E58BD63EC40BEC8 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8",
                "htilde":"1 0390695BBDE60D3C80592485EA4927A7AABF9FCA58B2B3C91181F8B77410BAE9 1 238CAB398C8033ACE7CE0159A7D19B720E46FDFD39FEF67509C2969C3A1AC8FF 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8",
                "h_cap":"1 03773EA38263225C820C86382E59191B9DA22EAD88AC9DE09CD06BB6443C33A3 1 1DE7F2A3E9525E86166EBB93B65519EBA689503E1A908890F6C3995B799355CA 1 0EEAC43B946C9C39880F0E734DCAF52648F898F538F3E0C32308781C1873F687 1 08525F1C036F893553295EFB09E69697DA39EAEA08501FA34327DC628553D785 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000",
                "u":"1 1EF4AAB0259D225250C6C16D9ECA177A0DE153C69A6E03E08470765DF8509481 1 00495E23E2350CBBF67130EA3ECC61AF6D729EF9AFDB34077D8BC254246B911D 1 145CC6577DA9F71C28ABAE9630E6D60CCD4379BB0316732C6F27945DA6EAE763 1 137B82272A33ECB263EB754BD809482F531D09C1B86E218B77661F4D85822525 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000",
                "pk":"1 01C61590137193095EE0D36417CF9043E810E90FC63577100B9A95E187044A30 1 05345DEAD330D2E11AD84137329706301723EC3DBF9962B91DAA0B0AEEAA6CD1 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8",
                "y":"1 19D03D2C2E2DD89AE9431BDFA4521DFAFDB09F98FF30725BBC73478626496341 1 0E5EFFB4162B221BAC2570F46B8F9317C44AC1986FA8EC5581F870A08442A1C9 1 0DE6FAECA1AD4A340DECF052BBDF735F43D601D33CDE092F85B16EB2410BB2A5 1 1C79C1D08C696D44829870817D025F67C75A89E273C6376548AEB333785934B4 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000"
            }
        })
    }

    pub fn build(did: &DidValue, schema_seq_no: u64) -> CredentialDefinitionV1 {
        CredentialDefinitionV1 {
            id: build_cred_def_id(did, schema_seq_no),
            schema_id: SchemaId(schema_seq_no.to_string()),
            signature_type: TYPE,
            tag: TAG.to_string(),
            value: serde_json::from_value(cred_def_data()).unwrap(),
        }
    }

    pub fn publish(
        identity: &Identity,
        pool: &TestPool,
        cred_def: CredentialDefinitionV1,
    ) -> CredentialDefinitionId {
        let cred_def_id = cred_def.id.clone();

        // Send Credential Definition
        let mut cred_def_request = pool
            .request_builder()
            .build_cred_def_request(
                &identity.did,
                CredentialDefinition::CredentialDefinitionV1(cred_def),
            )
            .unwrap();

        let _cred_def_response =
            sign_and_send_request(identity, pool, &mut cred_def_request).unwrap();

        cred_def_id
    }
}

pub mod revoc_reg {
    use super::*;
    use indy_vdr::ledger::identifiers::{CredentialDefinitionId, RevocationRegistryId};
    use indy_vdr::ledger::requests::rev_reg::{RevocationRegistryDelta, RevocationRegistryDeltaV1};
    use indy_vdr::ledger::requests::rev_reg_def::{
        IssuanceType, RegistryType, RevocationRegistryDefinition, RevocationRegistryDefinitionV1,
        RevocationRegistryDefinitionValue,
    };

    pub const TAG: &str = "tag";
    pub const REVOC_DEF_TYPE: RegistryType = RegistryType::CL_ACCUM;
    pub const FROM: i64 = 123456789;
    pub const TO: i64 = 987654321;

    pub fn build_revoc_reg_def_id(
        did: &DidValue,
        cred_def_id: &CredentialDefinitionId,
    ) -> RevocationRegistryId {
        RevocationRegistryId(format!(
            "{}:4:{}:{}:{}",
            did.0,
            cred_def_id.0,
            REVOC_DEF_TYPE.to_str(),
            TAG
        ))
    }

    pub fn revoc_reg_def_public_keys() -> serde_json::Value {
        json!({
            "accumKey": {
                "z": "1 1AC98E7E072E589AF80C32A5581CB2E33930D061AA0D01229B97543B7A3AAE15 1 1AA1D2CE753BF8A6D65F62DAF18AD623DEF09C20C3D24FEE4CF1A0562EBC8869 1 001D93DBE7607EFD3568DBB089D1620B940C68A66702ED4359C538919EFE2ACD 1 1F69F5D3A3B1B4611951508408867E074AA745E0B28F16A0C4416404D25AA768 1 06F2352D0802582E2621674286F560A517C14F864A4B80B2EF0C702CEC07799A 1 12520DE1478641BC1988DD132E6E40D2C63764887D2B9CB065DB30019D15A6FB 1 03558B526F4D29079C0100CEFEFDE828AFDDB9049064F03B9F09109D000C4595 1 1386BADC875AA325B863F3BFD675FFA31015BDA621C3A8263DCC874B5286CB15 1 160F45BF3FAD086D0DB2D9662323DB4ACE4774F3EA73A23C9ABF39560E998643 1 1C2C195466F98F7B164406DF544E5524057269D1FEF5687D19F2E603C1EF8689 1 1535FC9089E48BF74677E94C47212F47D503E2E6FFB6B26EF450C5EB6C7197A1 1 1470AC0494C6DD85F561F803A57A80EB34FFAE8A3468406505DAF55659003879"
            }
        })
    }

    pub fn revoc_reg_entry_value() -> serde_json::Value {
        json!({
            "accum": "1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000 1 0000000000000000000000000000000000000000000000000000000000000000"
        })
    }

    pub fn revoc_reg_delta() -> RevocationRegistryDelta {
        RevocationRegistryDelta::RevocationRegistryDeltaV1(RevocationRegistryDeltaV1 {
            value: serde_json::from_value(revoc_reg_entry_value()).unwrap(),
        })
    }

    pub fn revoc_reg_def_value() -> RevocationRegistryDefinitionValue {
        RevocationRegistryDefinitionValue {
            issuance_type: IssuanceType::ISSUANCE_BY_DEFAULT,
            max_cred_num: 5,
            public_keys: serde_json::from_value(revoc_reg_def_public_keys()).unwrap(),
            tails_hash: String::from("hash"),
            tails_location: String::from("path/to/tails"),
        }
    }

    pub fn build(
        did: &DidValue,
        cred_def_id: &CredentialDefinitionId,
    ) -> RevocationRegistryDefinitionV1 {
        RevocationRegistryDefinitionV1 {
            id: build_revoc_reg_def_id(did, cred_def_id),
            revoc_def_type: REVOC_DEF_TYPE,
            tag: TAG.to_string(),
            cred_def_id: cred_def_id.clone(),
            value: revoc_reg_def_value(),
        }
    }

    pub fn publish(
        identity: &Identity,
        pool: &TestPool,
        revoc_reg_def: RevocationRegistryDefinitionV1,
    ) -> RevocationRegistryId {
        let revoc_reg_def_id = revoc_reg_def.id.clone();

        // Send Revocation Registry Definition
        let mut revoc_reg_def_request = pool
            .request_builder()
            .build_revoc_reg_def_request(
                &identity.did,
                RevocationRegistryDefinition::RevocationRegistryDefinitionV1(revoc_reg_def),
            )
            .unwrap();

        let _cred_def_response =
            sign_and_send_request(identity, pool, &mut revoc_reg_def_request).unwrap();

        revoc_reg_def_id
    }
}

pub mod taa {
    use super::*;
    use indy_vdr::ledger::requests::author_agreement::{
        AcceptanceMechanisms, GetTxnAuthorAgreementData,
    };

    pub fn gen_aml_data() -> (AcceptanceMechanisms, String, String, String) {
        let aml_label = rand_string(30);

        let mut aml: AcceptanceMechanisms = AcceptanceMechanisms::new();
        aml.0.insert(aml_label.clone(), json!(rand_string(30)));

        let version: String = rand_version();
        let aml_context: String = rand_string(30);
        (aml, aml_label, version, aml_context)
    }

    pub fn gen_taa_data() -> (String, String, u64) {
        let text: String = rand_string(30);
        let version: String = rand_version();
        let ratification_ts = current_timestamp();
        (text, version, ratification_ts)
    }

    fn send_taa(
        pool: &TestPool,
        trustee: &Identity,
        taa_text: &str,
        taa_version: &str,
        ratification_ts: u64,
    ) -> String {
        let mut request = pool
            .request_builder()
            .build_txn_author_agreement_request(
                &trustee.did,
                Some(taa_text.to_string()),
                taa_version.to_string(),
                Some(ratification_ts),
                None,
            )
            .unwrap();

        sign_and_send_request(trustee, pool, &mut request).unwrap()
    }

    pub fn set_taa(pool: &TestPool, trustee: &Identity) -> (String, String, String, u64) {
        let (taa_text, taa_version, ratification_ts) = gen_taa_data();
        let response = send_taa(pool, trustee, &taa_text, &taa_version, ratification_ts);
        (response, taa_text, taa_version, ratification_ts)
    }

    pub fn disable_taa(pool: &TestPool, trustee: &Identity) {
        let mut request = pool
            .request_builder()
            .build_disable_all_txn_author_agreements_request(&trustee.did)
            .unwrap();
        let _response = sign_and_send_request(trustee, pool, &mut request).unwrap();
    }

    pub fn set_aml(
        pool: &TestPool,
        trustee: &Identity,
    ) -> (String, AcceptanceMechanisms, String, String, String) {
        let (aml, aml_label, aml_version, aml_context) = gen_aml_data();

        let mut request = pool
            .request_builder()
            .build_acceptance_mechanisms_request(
                &trustee.did,
                aml.clone(),
                aml_version.to_string(),
                Some(aml_context.clone()),
            )
            .unwrap();
        let response = sign_and_send_request(trustee, pool, &mut request).unwrap();

        (response, aml, aml_label, aml_version, aml_context)
    }

    pub fn get_taa(pool: &TestPool, txn_author_agreement_response: &str, version: &str) -> String {
        let data = GetTxnAuthorAgreementData {
            digest: None,
            version: Some(version.to_string()),
            timestamp: None,
        };

        let request = pool
            .request_builder()
            .build_get_txn_author_agreement_request(None, Some(&data))
            .unwrap();

        pool.send_request_with_retries(&request, txn_author_agreement_response)
            .unwrap()
    }

    pub fn check_taa(
        pool: &TestPool,
        txn_author_agreement_response: &str,
        version: &str,
        expected_data: serde_json::Value,
    ) {
        let get_txn_author_agreement_response =
            get_taa(pool, txn_author_agreement_response, version);

        let response: serde_json::Value =
            serde_json::from_str(&get_txn_author_agreement_response).unwrap();
        assert_eq!(response["result"]["data"]["text"], expected_data["text"]);
        assert_eq!(
            response["result"]["data"]["version"],
            expected_data["version"]
        );
        assert_eq!(
            response["result"]["data"]["ratification_ts"],
            expected_data["ratification_ts"]
        );
    }
}
