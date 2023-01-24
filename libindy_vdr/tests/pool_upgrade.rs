#[macro_use]
mod utils;

inject_dependencies!();

use indy_vdr::ledger::constants;
use indy_vdr::ledger::requests::pool::Schedule;
use indy_vdr::utils::did::DidValue;

use crate::utils::fixtures::*;
use crate::utils::helpers;

fn _empty_schedule() -> Schedule {
    Schedule::new()
}

fn _schedule() -> Schedule {
    let next_year = time::OffsetDateTime::now_utc().year() + 1;

    let mut schedule = Schedule::new();
    schedule.insert(
        String::from("Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv"),
        format!("{}-01-25T12:49:05.258870+00:00", next_year),
    );
    schedule.insert(
        String::from("8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"),
        format!("{}-01-25T13:49:05.258870+00:00", next_year),
    );
    schedule.insert(
        String::from("DKVxG2fXXTU8yT5N7hGEbXB3dfdAnYv1JczDUHpmDxya"),
        format!("{}-01-25T14:49:05.258870+00:00", next_year),
    );
    schedule.insert(
        String::from("4PS3EDQ3dW1tci1Bp6543CfuuebjFrg36kLAUcskGfaA"),
        format!("{}-01-25T15:49:05.258870+00:00", next_year),
    );

    schedule
}

const NAME: &str = "test-upgrade-libindy";
const VERSION: &str = "2.0.0";
const SHA256: &str = "f284bdc3c1c9e24a494e285cb387c69510f28de51c15bb93179d9c7f28705398";
const START: &str = "start";
const CANCEL: &str = "cancel";
const PACKAGE: &str = "some_package";
const JUSTIFICATION: &str = "Upgrade is not required";

#[test]
fn empty() {
    // Empty test to run module
}

#[cfg(test)]
mod builder {
    use super::*;
    use indy_vdr::ledger::RequestBuilder;

    mod pool_config {
        use super::*;

        #[rstest]
        fn test_build_pool_upgrade_requests_for_start_action(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_pool_upgrade_request(
                    &trustee_did,
                    NAME,
                    VERSION,
                    START,
                    SHA256,
                    None,
                    Some(_empty_schedule()),
                    None,
                    false,
                    false,
                    None,
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::POOL_UPGRADE,
                "name": NAME,
                "version": VERSION,
                "action": START,
                "sha256": SHA256,
                "schedule": {},
                "reinstall": false,
                "force": false
            });
            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_pool_upgrade_requests_for_cancel_action(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_pool_upgrade_request(
                    &trustee_did,
                    NAME,
                    VERSION,
                    CANCEL,
                    SHA256,
                    None,
                    None,
                    Some(JUSTIFICATION),
                    false,
                    false,
                    None,
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::POOL_UPGRADE,
                "name": NAME,
                "version": VERSION,
                "action": CANCEL,
                "sha256": SHA256,
                "justification": JUSTIFICATION,
                "reinstall": false,
                "force": false
            });
            helpers::check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_pool_upgrade_requests_for_package(
            request_builder: RequestBuilder,
            trustee_did: DidValue,
        ) {
            let request = request_builder
                .build_pool_upgrade_request(
                    &trustee_did,
                    NAME,
                    VERSION,
                    START,
                    SHA256,
                    None,
                    Some(_empty_schedule()),
                    None,
                    false,
                    false,
                    Some(PACKAGE),
                )
                .unwrap();

            let expected_operation = json!({
                "type": constants::POOL_UPGRADE,
                "name": NAME,
                "version": VERSION,
                "action": START,
                "sha256": SHA256,
                "schedule": {},
                "reinstall": false,
                "force": false,
                "package": PACKAGE
            });
            helpers::check_request_operation(&request, expected_operation);
        }
    }
}

#[cfg(test)]
#[cfg(feature = "local_nodes_pool")]
mod send_pool_upgrade {
    use super::*;
    use crate::utils::crypto::Identity;
    use crate::utils::pool::TestPool;

    #[rstest]
    fn test_pool_send_pool_upgrade_request(pool: TestPool, trustee: Identity) {
        // Schedule Pool Upgrade
        let mut request = pool
            .request_builder()
            .build_pool_upgrade_request(
                &trustee.did,
                NAME,
                VERSION,
                START,
                SHA256,
                None,
                Some(_schedule()),
                None,
                false,
                false,
                None,
            )
            .unwrap();

        trustee.sign_request(&mut request);

        let _response = pool.send_full_request(&request, None, None).unwrap();

        // Cancel Pool Upgrade
        let mut request = pool
            .request_builder()
            .build_pool_upgrade_request(
                &trustee.did,
                NAME,
                VERSION,
                CANCEL,
                SHA256,
                None,
                None,
                Some(JUSTIFICATION),
                false,
                false,
                None,
            )
            .unwrap();

        trustee.sign_request(&mut request);

        let _response = pool.send_full_request(&request, None, None).unwrap();
    }
}
