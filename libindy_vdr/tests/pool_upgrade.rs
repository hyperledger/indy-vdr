#[macro_use]
mod utils;

extern crate chrono;

inject_dependencies!();

use indy_vdr::common::did::DidValue;
use indy_vdr::ledger::constants;
use chrono::prelude::*;

use crate::utils::fixtures::*;
use crate::utils::pool::TestPool;
use crate::utils::crypto::Identity;
use indy_vdr::ledger::requests::pool::Schedule;

#[test]
fn empty() {
    // Empty test to run module
}

fn _empty_schedule() -> Schedule { Schedule::new() }

fn _name() -> &'static str { "test-upgrade-libindy" }

fn _version() -> &'static str { "2.0.0" }

fn _sha256() -> &'static str { "f284bdc3c1c9e24a494e285cb387c69510f28de51c15bb93179d9c7f28705398" }

fn _start() -> &'static str { "start" }

fn _cancel() -> &'static str { "cancel" }

fn _package() -> &'static str { "some_package" }

fn _justification() -> &'static str { "Upgrade is not required" }

fn _schedule() -> Schedule {
    let next_year = Utc::now().year() + 1;

    let mut schedule = Schedule::new();
    schedule.insert(String::from("Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv"), format!("{}-01-25T12:49:05.258870+00:00", next_year));
    schedule.insert(String::from("8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"), format!("{}-01-25T13:49:05.258870+00:00", next_year));
    schedule.insert(String::from("DKVxG2fXXTU8yT5N7hGEbXB3dfdAnYv1JczDUHpmDxya"), format!("{}-01-25T14:49:05.258870+00:00", next_year));
    schedule.insert(String::from("4PS3EDQ3dW1tci1Bp6543CfuuebjFrg36kLAUcskGfaA"), format!("{}-01-25T15:49:05.258870+00:00", next_year));

    schedule
}

#[cfg(test)]
mod builder {
    use super::*;
    use indy_vdr::ledger::RequestBuilder;

    mod pool_config {
        use super::*;
        use crate::utils::helpers::check_request_operation;

        #[rstest]
        fn test_build_pool_upgrade_requests_for_start_action(request_builder: RequestBuilder,
                                                             trustee_did: DidValue) {
            let request =
                request_builder
                    .build_pool_upgrade(&trustee_did,
                                        _name(),
                                        _version(),
                                        _start(),
                                        _sha256(),
                                        None,
                                        Some(_empty_schedule()),
                                        None,
                                        false,
                                        false,
                                        None).unwrap();

            let expected_operation = json!({
                "type": constants::POOL_UPGRADE,
                "name": _name(),
                "version": _version(),
                "action": _start(),
                "sha256": _sha256(),
                "schedule": {},
                "reinstall": false,
                "force": false
            });
            check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_pool_upgrade_requests_for_cancel_action(request_builder: RequestBuilder,
                                                              trustee_did: DidValue) {
            let request =
                request_builder
                    .build_pool_upgrade(&trustee_did,
                                        _name(),
                                        _version(),
                                        _cancel(),
                                        _sha256(),
                                        None,
                                        None,
                                        Some(_justification()),
                                        false,
                                        false,
                                        None).unwrap();

            let expected_operation = json!({
                "type": constants::POOL_UPGRADE,
                "name": _name(),
                "version": _version(),
                "action": _cancel(),
                "sha256": _sha256(),
                "justification": _justification(),
                "reinstall": false,
                "force": false
            });
            check_request_operation(&request, expected_operation);
        }

        #[rstest]
        fn test_build_pool_upgrade_requests_for_package(request_builder: RequestBuilder,
                                                        trustee_did: DidValue) {
            let request =
                request_builder
                    .build_pool_upgrade(&trustee_did,
                                        _name(),
                                        _version(),
                                        _start(),
                                        _sha256(),
                                        None,
                                        Some(_empty_schedule()),
                                        None,
                                        false,
                                        false,
                                        Some(_package())).unwrap();

            let expected_operation = json!({
                "type": constants::POOL_UPGRADE,
                "name": _name(),
                "version": _version(),
                "action": _start(),
                "sha256": _sha256(),
                "schedule": {},
                "reinstall": false,
                "force": false,
                "package": _package()
            });
            check_request_operation(&request, expected_operation);
        }
    }
}

#[cfg(test)]
mod send_pool_upgrade {
    use super::*;
    use crate::utils::helpers;

    #[rstest]
    fn test_pool_send_pool_upgrade_request(pool: TestPool, trustee: Identity) {
        // Schedule Pool Upgrade
        let mut request =
            pool.request_builder()
                .build_pool_upgrade(&trustee.did,
                                    _name(),
                                    _version(),
                                    _start(),
                                    _sha256(),
                                    None,
                                    Some(_schedule()),
                                    None,
                                    false,
                                    false,
                                    None).unwrap();

        let _response = helpers::sign_and_send_request(&trustee, &pool, &mut request).unwrap();

        // Cancel Pool Upgrade
        let mut request =
            pool.request_builder()
                .build_pool_upgrade(&trustee.did,
                                    _name(),
                                    _version(),
                                    _cancel(),
                                    _sha256(),
                                    None,
                                    None,
                                    Some(_justification()),
                                    false,
                                    false,
                                    None).unwrap();

        let _response = helpers::sign_and_send_request(&trustee, &pool, &mut request).unwrap();
    }
}