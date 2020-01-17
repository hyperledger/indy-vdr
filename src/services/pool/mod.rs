extern crate rand;
extern crate rmp_serde;
extern crate time;

mod genesis;
mod networker;
mod pool;
mod requests;
mod state_proof;
mod types;

pub use networker::{Networker, ZMQNetworker};
pub use pool::{
    perform_get_txn, perform_get_txn_consensus, perform_get_txn_full, perform_refresh, Pool,
};
pub use types::PoolConfig;

use crate::domain::ledger::txn;
pub use txn::LedgerType; // temporary for HTTP client

#[cfg(test)]
mod tests {
    // use std::thread;

    // use crate::domain::ledger::request::ProtocolVersion;
    // use crate::services::pool::types::*;
    // use crate::utils::test;

    // use super::*;

    // const TEST_PROTOCOL_VERSION: usize = 2;

    /*fn _set_protocol_version(version: usize) {
        ProtocolVersion::set(version);
    }*/

    /*
    mod pool_service {
        use std::path;

        use libc::c_char;

        use indy_api_types::{ErrorCode, INVALID_POOL_HANDLE};

        use super::*;

        #[test]
        fn pool_service_new_works() {
            PoolService::new();
            assert!(true, "No crashes on PoolService::new");
        }

        #[test]
        fn pool_service_drop_works() {
            fn drop_test() {
                PoolService::new();
            }

            drop_test();
            assert!(true, "No crashes on PoolService::drop");
        }

        #[test]
        fn pool_service_close_works() {
            test::cleanup_storage("pool_service_close_works");

            let ps = PoolService::new();
            let pool_id = next_pool_handle();
            let (send_cmd_sock, recv_cmd_sock) =
                pool_create_pair_of_sockets("pool_service_close_works");
            ps.open_pools.borrow_mut().insert(
                pool_id,
                ZMQPool::new(
                    Pool::new("", pool_id, PoolOpenConfig::default()),
                    send_cmd_sock,
                ),
            );
            let cmd_id = ps.close(pool_id).unwrap();
            let recv = recv_cmd_sock.recv_multipart(zmq::DONTWAIT).unwrap();
            assert_eq!(recv.len(), 3);
            assert_eq!(COMMAND_EXIT, String::from_utf8(recv[0].clone()).unwrap());
            assert_eq!(cmd_id, LittleEndian::read_i32(recv[1].as_slice()));
        }

        #[test]
        fn pool_service_refresh_works() {
            test::cleanup_storage("pool_service_refresh_works");

            let ps = PoolService::new();
            let pool_id = next_pool_handle();
            let (send_cmd_sock, recv_cmd_sock) =
                pool_create_pair_of_sockets("pool_service_refresh_works");
            ps.open_pools.borrow_mut().insert(
                pool_id,
                ZMQPool::new(
                    Pool::new("", pool_id, PoolOpenConfig::default()),
                    send_cmd_sock,
                ),
            );
            let cmd_id = ps.refresh(pool_id).unwrap();
            let recv = recv_cmd_sock.recv_multipart(zmq::DONTWAIT).unwrap();
            assert_eq!(recv.len(), 3);
            assert_eq!(COMMAND_REFRESH, String::from_utf8(recv[0].clone()).unwrap());
            assert_eq!(cmd_id, LittleEndian::read_i32(recv[1].as_slice()));
        }

        #[test]
        fn pool_service_delete_works() {
            test::cleanup_storage("pool_service_delete_works");

            let ps = PoolService::new();
            let pool_name = "pool_service_delete_works";
            let path: path::PathBuf = environment::pool_path(pool_name);
            fs::create_dir_all(path.as_path()).unwrap();
            assert!(path.exists());
            ps.delete(pool_name).unwrap();
            assert!(!path.exists());

            test::cleanup_storage("pool_service_delete_works");
        }

        #[test]
        fn pool_service_delete_works_for_opened() {
            test::cleanup_storage("pool_service_delete_works_for_opened");

            let (send_cmd_sock, _recv_cmd_sock) =
                pool_create_pair_of_sockets("pool_service_delete_works_for_opened");
            let ps = PoolService::new();
            let pool_name = "pool_service_delete_works_for_opened";
            let path: path::PathBuf = environment::pool_path(pool_name);
            let pool_id = next_pool_handle();

            let pool = Pool::new(pool_name, pool_id, PoolOpenConfig::default());
            ps.open_pools
                .borrow_mut()
                .insert(pool_id, ZMQPool::new(pool, send_cmd_sock));

            fs::create_dir_all(path.as_path()).unwrap();
            assert!(path.exists());
            let res = ps.delete(pool_name);
            assert_eq!(LedgerErrorKind::InvalidState, res.unwrap_err().kind());
            assert!(path.exists());

            test::cleanup_storage("pool_service_delete_works_for_opened");
        }

        #[test]
        fn pool_send_tx_works() {
            test::cleanup_storage("pool_send_tx_works");

            let name = "test";
            let (send_cmd_sock, recv_cmd_sock) = pool_create_pair_of_sockets("pool_send_tx_works");
            let pool_id = next_pool_handle();
            let pool = Pool::new(name, pool_id, PoolOpenConfig::default());
            let ps = PoolService::new();
            ps.open_pools
                .borrow_mut()
                .insert(pool_id, ZMQPool::new(pool, send_cmd_sock));
            let test_data = "str_instead_of_tx_json";
            ps.send_tx(pool_id, test_data).unwrap();
            assert_eq!(
                recv_cmd_sock.recv_string(zmq::DONTWAIT).unwrap().unwrap(),
                test_data
            );
        }

        #[test]
        fn pool_send_tx_works_for_closed_socket() {
            test::cleanup_storage("pool_send_tx_works_for_closed_socket");

            let name = "test";
            let zmq_ctx = zmq::Context::new();
            let send_cmd_sock = zmq_ctx.socket(zmq::SocketType::PAIR).unwrap();

            let pool_id = next_pool_handle();
            let pool = Pool::new(name, pool_id, PoolOpenConfig::default());
            let ps = PoolService::new();
            ps.open_pools
                .borrow_mut()
                .insert(pool_id, ZMQPool::new(pool, send_cmd_sock));
            let res = ps.send_tx(pool_id, "test_data");
            assert_eq!(LedgerErrorKind::IOError, res.unwrap_err().kind());
        }

        #[test]
        fn pool_send_tx_works_for_invalid_handle() {
            test::cleanup_storage("pool_send_tx_works_for_invalid_handle");
            let ps = PoolService::new();
            let res = ps.send_tx(INVALID_POOL_HANDLE, "txn");
            assert_eq!(LedgerErrorKind::InvalidPoolHandle, res.unwrap_err().kind());
        }

        #[test]
        fn pool_send_action_works() {
            test::cleanup_storage("pool_send_action_works");

            let (send_cmd_sock, recv_cmd_sock) =
                pool_create_pair_of_sockets("pool_send_action_works");
            let pool_id = next_pool_handle();
            let pool = Pool::new("pool_send_action_works", pool_id, PoolOpenConfig::default());
            let ps = PoolService::new();
            ps.open_pools
                .borrow_mut()
                .insert(pool_id, ZMQPool::new(pool, send_cmd_sock));
            let test_data = "str_instead_of_tx_json";
            ps.send_action(pool_id, test_data, None, None).unwrap();
            assert_eq!(
                recv_cmd_sock.recv_string(zmq::DONTWAIT).unwrap().unwrap(),
                test_data
            );
        }

        #[test]
        fn pool_close_works_for_invalid_handle() {
            test::cleanup_storage("pool_close_works_for_invalid_handle");
            let ps = PoolService::new();
            let res = ps.close(INVALID_POOL_HANDLE);
            assert_eq!(LedgerErrorKind::InvalidPoolHandle, res.unwrap_err().kind());
        }

        #[test]
        fn pool_refresh_works_for_invalid_handle() {
            test::cleanup_storage("pool_refresh_works_for_invalid_handle");
            let ps = PoolService::new();
            let res = ps.refresh(INVALID_POOL_HANDLE);
            assert_eq!(LedgerErrorKind::InvalidPoolHandle, res.unwrap_err().kind());
        }

        #[test]
        fn pool_register_sp_parser_works() {
            test::cleanup_storage("pool_register_sp_parser_works");
            REGISTERED_SP_PARSERS.lock().unwrap().clear();
            extern "C" fn test_sp(
                _reply_from_node: *const c_char,
                _parsed_sp: *mut *const c_char,
            ) -> ErrorCode {
                ErrorCode::Success
            }
            extern "C" fn test_free(_data: *const c_char) -> ErrorCode {
                ErrorCode::Success
            }
            PoolService::register_sp_parser("test", test_sp, test_free).unwrap();
        }

        #[test]
        fn pool_get_sp_parser_works() {
            test::cleanup_storage("pool_get_sp_parser_works");
            REGISTERED_SP_PARSERS.lock().unwrap().clear();
            extern "C" fn test_sp(
                _reply_from_node: *const c_char,
                _parsed_sp: *mut *const c_char,
            ) -> ErrorCode {
                ErrorCode::Success
            }
            extern "C" fn test_free(_data: *const c_char) -> ErrorCode {
                ErrorCode::Success
            }
            PoolService::register_sp_parser("test", test_sp, test_free).unwrap();
            PoolService::get_sp_parser("test").unwrap();
        }

        #[test]
        fn pool_get_sp_parser_works_for_invalid_name() {
            test::cleanup_storage("pool_get_sp_parser_works_for_invalid_name");
            REGISTERED_SP_PARSERS.lock().unwrap().clear();
            assert_eq!(None, PoolService::get_sp_parser("test"));
        }

        #[test]
        pub fn pool_add_open_pool_works() {
            test::cleanup_storage("pool_add_open_pool_works");
            let ps = PoolService::new();
            let (send_cmd_sock, _recv_cmd_sock) =
                pool_create_pair_of_sockets("pool_add_open_pool_works");
            let pool_id = next_pool_handle();
            let pool = Pool::new(
                "pool_add_open_pool_works",
                pool_id,
                PoolOpenConfig::default(),
            );
            ps.pending_pools
                .borrow_mut()
                .insert(pool_id, ZMQPool::new(pool, send_cmd_sock));
            assert_match!(Ok(_pool_id), ps.add_open_pool(pool_id));
        }

        #[test]
        pub fn pool_add_open_pool_works_for_no_pending_pool() {
            test::cleanup_storage("pool_add_open_pool_works_for_no_pending_pool");
            let ps = PoolService::new();
            let res = ps.add_open_pool(INVALID_POOL_HANDLE);
            assert_eq!(LedgerErrorKind::InvalidPoolHandle, res.unwrap_err().kind());
        }
    }
    */

    /*#[test]
    fn pool_drop_works_for_after_close() {
        use crate::utils::test;
        use std::time;

        test::cleanup_storage("pool_drop_works_for_after_close");

        fn drop_test() {
            _set_protocol_version(TEST_PROTOCOL_VERSION);
            let ps = PoolService::new();

            let pool_name = "pool_drop_works_for_after_close";
            let gen_txn = test::gen_txns()[0].clone();

            let (send_cmd_sock, recv_cmd_sock) = pool_create_pair_of_sockets("drop_test");

            // create minimal fs config stub before Pool::new()
            let mut pool_path = environment::pool_path(pool_name);
            fs::create_dir_all(&pool_path).unwrap();
            pool_path.push(pool_name);
            pool_path.set_extension("txn");
            let mut file = fs::File::create(pool_path).unwrap();
            file.write(&gen_txn.as_bytes()).unwrap();

            let pool_id = next_pool_handle();
            let mut pool = Pool::new(pool_name, pool_id, PoolOpenConfig::default());
            pool.work(recv_cmd_sock);
            ps.open_pools
                .borrow_mut()
                .insert(pool_id, ZMQPool::new(pool, send_cmd_sock));
            thread::sleep(time::Duration::from_secs(1));
            ps.close(pool_id).unwrap();
            thread::sleep(time::Duration::from_secs(1));
        }

        drop_test();
        test::cleanup_storage("pool_drop_works_for_after_close");
    }*/

    /*
    pub mod nodes_emulator {
        use crate::utils::base58::{FromBase58, ToBase58};
        use crate::utils::crypto;

        use super::*;

        use crate::services::pool::request_handler::DEFAULT_GENERATOR;
        use ursa::bls::{Generator, SignKey, VerKey};

        pub static POLL_TIMEOUT: i64 = 1_000; /* in ms */

        pub fn node() -> NodeTransactionV1 {
            let blskey = VerKey::new(
                &Generator::from_bytes(&DEFAULT_GENERATOR.from_base58().unwrap()).unwrap(),
                &SignKey::new(None).unwrap(),
            )
            .unwrap()
            .as_bytes()
            .to_base58();

            NodeTransactionV1 {
                txn: Txn {
                    txn_type: "1".to_string(),
                    protocol_version: None,
                    data: TxnData {
                        data: NodeData {
                            alias: "n1".to_string(),
                            client_ip: Some("127.0.0.1".to_string()),
                            client_port: Some(9000),
                            node_ip: Some(String::new()),
                            node_port: Some(0),
                            services: Some(vec!["VALIDATOR".to_string()]),
                            blskey: Some(blskey.to_string()),
                            blskey_pop: None,
                        },
                        dest: "Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv".to_string(),
                        verkey: None,
                    },
                    metadata: TxnMetadata {
                        req_id: None,
                        from: String::new(),
                    },
                },
                txn_metadata: Metadata {
                    creation_time: None,
                    seq_no: None,
                    txn_id: None,
                },
                req_signature: ReqSignature {
                    type_: None,
                    values: None,
                },
                ver: String::new(),
            }
        }

        pub fn node_2() -> NodeTransactionV1 {
            let blskey = VerKey::new(
                &Generator::from_bytes(&DEFAULT_GENERATOR.from_base58().unwrap()).unwrap(),
                &SignKey::new(None).unwrap(),
            )
            .unwrap()
            .as_bytes()
            .to_base58();

            NodeTransactionV1 {
                txn: Txn {
                    txn_type: "1".to_string(),
                    protocol_version: None,
                    data: TxnData {
                        data: NodeData {
                            alias: "n2".to_string(),
                            client_ip: Some("127.0.0.1".to_string()),
                            client_port: Some(9001),
                            node_ip: Some(String::new()),
                            node_port: Some(0),
                            services: Some(vec!["VALIDATOR".to_string()]),
                            blskey: Some(blskey),
                            blskey_pop: None,
                        },
                        dest: "Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv".to_string(),
                        verkey: None,
                    },
                    metadata: TxnMetadata {
                        req_id: None,
                        from: String::new(),
                    },
                },
                txn_metadata: Metadata {
                    creation_time: None,
                    seq_no: None,
                    txn_id: None,
                },
                req_signature: ReqSignature {
                    type_: None,
                    values: None,
                },
                ver: String::new(),
            }
        }

        pub fn start(gt: &mut NodeTransactionV1) -> zmq::Socket {
            let keypair = crypto::gen_keypair().unwrap();
            let (pkc, skc) = (
                crypto::vk_to_curve25519(keypair.public).expect("Invalid pkc"),
                crypto::sk_to_curve25519(keypair.secret).expect("Invalid skc"),
            );
            let ctx = zmq::Context::new();
            let s: zmq::Socket = ctx.socket(zmq::SocketType::ROUTER).unwrap();

            gt.txn.data.dest = keypair.public.to_bytes().to_base58();

            s.set_curve_publickey(&zmq::z85_encode(&pkc[..]).unwrap().as_bytes())
                .expect("set public key");
            s.set_curve_secretkey(&zmq::z85_encode(&skc[..]).unwrap().as_bytes())
                .expect("set secret key");
            s.set_curve_server(true).expect("set curve server");

            s.bind("tcp://127.0.0.1:*").expect("bind");

            let parts = s.get_last_endpoint().unwrap().unwrap();
            let parts = parts.rsplit(":").collect::<Vec<&str>>();

            gt.txn.data.data.client_port = Some(parts[0].parse::<u64>().unwrap());

            s
        }

        pub fn next(s: &zmq::Socket) -> Option<String> {
            let poll_res = s.poll(zmq::POLLIN, POLL_TIMEOUT).expect("poll");
            if poll_res == 1 {
                let v = s.recv_multipart(zmq::DONTWAIT).expect("recv mulp");
                trace!("Node emulator poll recv {:?}", v);
                s.send_multipart(&[v[0].as_slice(), "po".as_bytes()], zmq::DONTWAIT)
                    .expect("send mulp");
                Some(String::from_utf8(v[1].clone()).unwrap())
            } else {
                warn!("Node emulator poll return {}", poll_res);
                None
            }
        }
    }
    */
}
