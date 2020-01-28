// use crate::common::error::prelude::*;

/*
- indy_vdr_get_last_error
- indy_vdr_set_protocol_version (for new pools) -> error code
- indy_vdr_set_config (for new pools) -> error code
- indy_vdr_set_logger(callback) -> error code
- indy_vdr_pool_create_from_transactions(char[], *pool_handle) -> error code
- indy_vdr_pool_create_from_genesis_file(char[]) -> error code
- indy_vdr_pool_get_transactions(pool_handle, char[]*) -> error code
- indy_vdr_pool_refresh(pool_handle, callback(command_handle, err, new_txns)) -> error code
- indy_vdr_pool_free(pool_handle) -> void
    (^ no more requests allowed on this pool, but existing ones may be completed)
- indy_vdr_pool_build_{nym, schema, etc}_request(pool_handle, ..., *request_handle) -> error code
- indy_vdr_pool_build_custom_request(pool_handle, char[] json, *request_handle) -> error code
- indy_vdr_request_submit(request_handle, callback(command_handle, err, result_json)) -> error code
- indy_vdr_request_submit_action(request_handle, nodes, timeout, callback(command_handle, err, result_json)) -> error code
- indy_vdr_request_free(request_handle) -> void
    (^ only needed for a request that isn't submitted)
- indy_vdr_request_get_body(request_handle, *char[]) -> error code
- indy_vdr_request_get_signature_input(request_handle, *char[]) -> error code
- indy_vdr_request_set_signature(request_handle, *char[]) -> error code
- indy_vdr_request_add_multi_signature(request_handle, *char[]) -> error code
*/
