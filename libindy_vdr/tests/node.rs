#[macro_use]
mod utils;

inject_dependencies!();

use indy_vdr::ledger::constants;
use indy_vdr::ledger::requests::node::{NodeOperationData, Services};
use indy_vdr::utils::did::DidValue;

use crate::utils::fixtures::*;

fn _dest() -> DidValue {
    DidValue("FYmoFw55GeQH7SRFa37dkx1d2dZ3zUF8ckg7wmL7ofN4".to_string())
}

fn _node_data() -> NodeOperationData {
    NodeOperationData {
        node_ip: Some(String::from("10.0.0.100")),
        node_port: Some(2),
        client_ip: Some(String::from("10.0.0.100")),
        client_port: Some(2),
        alias: String::from("Node5"),
        services: Some(vec![Services::VALIDATOR]),
        blskey: Some(String::from("4N8aUNHSgjQVgkpm8nhNEfDf6txHznoYREg9kirmJrkivgL4oSEimFF6nsQ6M41QvhM2Z33nves5vfSn9n1UwNFJBYtWVnHYMATn76vLuL3zU88KyeAYcHfsih3He6UHcXDxcaecHVz6jhCYz1P2UZn2bDVruL5wXpehgBfBaLKm3Ba")),
        blskey_pop: Some(String::from("RahHYiCvoNCtPTrVtP7nMC5eTYrsUA8WjXbdhNc8debh1agE9bGiJxWBXYNFbnJXoXhWFMvyqhqhRoq737YQemH5ik9oL7R4NTTCz2LEZhkgLJzB3QRQqJyBNyv7acbdHrAT8nQ9UkLbaVL9NBpnWXBTw4LEMePaSHEw66RzPNdAX1")),
    }
}

#[test]
fn empty() {
    // Empty test to run module
}

#[cfg(test)]
mod builder {
    use super::*;
    use indy_vdr::ledger::RequestBuilder;

    mod node {
        use super::*;
        use crate::utils::helpers::check_request_operation;

        #[rstest]
        fn test_build_node_request(request_builder: RequestBuilder, steward_did: DidValue) {
            let request = request_builder
                .build_node_request(&steward_did, &_dest(), _node_data())
                .unwrap();

            let expected_operation = json!({
                "type": constants::NODE,
                "dest": _dest(),
                "data": _node_data(),
            });

            check_request_operation(&request, expected_operation);
        }
    }
}
