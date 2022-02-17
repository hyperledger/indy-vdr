#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>


#define DEFAULT_ACK_TIMEOUT 20

#define DEFAULT_CONN_ACTIVE_TIMEOUT 5

#define DEFAULT_CONN_REQUEST_LIMIT 5

#define DEFAULT_FRESHNESS_TIMEOUT 300

#define DEFAULT_REPLY_TIMEOUT 60

#define DEFAULT_REQUEST_READ_NODES 2

enum ErrorCode
#ifdef __cplusplus
  : int64_t
#endif // __cplusplus
 {
  Success = 0,
  Config = 1,
  Connection = 2,
  FileSystem = 3,
  Input = 4,
  Resource = 5,
  Unavailable = 6,
  Unexpected = 7,
  Incompatible = 8,
  PoolNoConsensus = 30,
  PoolRequestFailed = 31,
  PoolTimeout = 32,
};
#ifndef __cplusplus
typedef int64_t ErrorCode;
#endif // __cplusplus

typedef int64_t CallbackId;
typedef int64_t PoolHandle;
typedef int64_t RequestHandle;
typedef const char *FfiStr;

typedef struct {
    int64_t len;
    uint8_t *data; // note: nullable
} ByteBuffer;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

ErrorCode indy_vdr_build_acceptance_mechanisms_request(FfiStr submitter_did,
                                                       FfiStr aml,
                                                       FfiStr version,
                                                       FfiStr aml_context,
                                                       RequestHandle *handle_p);

ErrorCode indy_vdr_build_attrib_request(FfiStr submitter_did,
                                        FfiStr target_did,
                                        FfiStr hash,
                                        FfiStr raw,
                                        FfiStr enc,
                                        RequestHandle *handle_p);

ErrorCode indy_vdr_build_cred_def_request(FfiStr submitter_did,
                                          FfiStr cred_def,
                                          RequestHandle *handle_p);

ErrorCode indy_vdr_build_custom_request(FfiStr request_json, RequestHandle *handle_p);

ErrorCode indy_vdr_build_disable_all_txn_author_agreements_request(FfiStr submitter_did,
                                                                   RequestHandle *handle_p);

ErrorCode indy_vdr_build_get_acceptance_mechanisms_request(FfiStr submitter_did,
                                                           int64_t timestamp,
                                                           FfiStr version,
                                                           RequestHandle *handle_p);

ErrorCode indy_vdr_build_get_attrib_request(FfiStr submitter_did,
                                            FfiStr target_did,
                                            FfiStr raw,
                                            FfiStr hash,
                                            FfiStr enc,
                                            RequestHandle *handle_p);

ErrorCode indy_vdr_build_get_cred_def_request(FfiStr submitter_did,
                                              FfiStr cred_def_id,
                                              RequestHandle *handle_p);

ErrorCode indy_vdr_build_get_nym_request(FfiStr submitter_did,
                                         FfiStr dest,
                                         RequestHandle *handle_p);

ErrorCode indy_vdr_build_get_revoc_reg_def_request(FfiStr submitter_did,
                                                   FfiStr revoc_reg_id,
                                                   RequestHandle *handle_p);

ErrorCode indy_vdr_build_get_revoc_reg_delta_request(FfiStr submitter_did,
                                                     FfiStr revoc_reg_id,
                                                     int64_t from_ts,
                                                     int64_t to_ts,
                                                     RequestHandle *handle_p);

ErrorCode indy_vdr_build_get_revoc_reg_request(FfiStr submitter_did,
                                               FfiStr revoc_reg_id,
                                               int64_t timestamp,
                                               RequestHandle *handle_p);

#if defined(DEFINE_RICH_SCHEMA)
ErrorCode indy_vdr_build_get_rich_schema_object_by_id_request(FfiStr submitter_did,
                                                              FfiStr rs_id,
                                                              RequestHandle *handle_p);
#endif

#if defined(DEFINE_RICH_SCHEMA)
ErrorCode indy_vdr_build_get_rich_schema_object_by_metadata_request(FfiStr submitter_did,
                                                                    FfiStr rs_type,
                                                                    FfiStr rs_name,
                                                                    FfiStr rs_version,
                                                                    RequestHandle *handle_p);
#endif

ErrorCode indy_vdr_build_get_schema_request(FfiStr submitter_did,
                                            FfiStr schema_id,
                                            RequestHandle *handle_p);

ErrorCode indy_vdr_build_get_txn_author_agreement_request(FfiStr submitter_did,
                                                          FfiStr data,
                                                          RequestHandle *handle_p);

ErrorCode indy_vdr_build_get_txn_request(FfiStr submitter_did,
                                         int32_t ledger_type,
                                         int32_t seq_no,
                                         RequestHandle *handle_p);

ErrorCode indy_vdr_build_get_validator_info_request(FfiStr submitter_did, RequestHandle *handle_p);

ErrorCode indy_vdr_build_nym_request(FfiStr submitter_did,
                                     FfiStr dest,
                                     FfiStr verkey,
                                     FfiStr alias,
                                     FfiStr role,
                                     RequestHandle *handle_p);

ErrorCode indy_vdr_build_revoc_reg_def_request(FfiStr submitter_did,
                                               FfiStr revoc_reg_def,
                                               RequestHandle *handle_p);

ErrorCode indy_vdr_build_revoc_reg_entry_request(FfiStr submitter_did,
                                                 FfiStr revoc_reg_def_id,
                                                 FfiStr revoc_reg_def_type,
                                                 FfiStr revoc_reg_entry,
                                                 RequestHandle *handle_p);

#if defined(DEFINE_RICH_SCHEMA)
ErrorCode indy_vdr_build_rich_schema_request(FfiStr submitter_did,
                                             FfiStr rs_id,
                                             FfiStr rs_content,
                                             FfiStr rs_name,
                                             FfiStr rs_version,
                                             FfiStr rs_type,
                                             FfiStr ver,
                                             RequestHandle *handle_p);
#endif

ErrorCode indy_vdr_build_schema_request(FfiStr submitter_did,
                                        FfiStr schema,
                                        RequestHandle *handle_p);

ErrorCode indy_vdr_build_txn_author_agreement_request(FfiStr submitter_did,
                                                      FfiStr text,
                                                      FfiStr version,
                                                      int64_t ratification_ts,
                                                      int64_t retirement_ts,
                                                      RequestHandle *handle_p);

ErrorCode indy_vdr_get_current_error(const char **error_json_p);

ErrorCode indy_vdr_pool_close(PoolHandle pool_handle);

ErrorCode indy_vdr_pool_create(FfiStr params, PoolHandle *handle_p);

ErrorCode indy_vdr_pool_get_status(PoolHandle pool_handle,
                                   void (*cb)(CallbackId cb_id, ErrorCode err, const char *response),
                                   CallbackId cb_id);

ErrorCode indy_vdr_pool_get_transactions(PoolHandle pool_handle,
                                         void (*cb)(CallbackId cb_id, ErrorCode err, const char *response),
                                         CallbackId cb_id);

ErrorCode indy_vdr_pool_get_verifiers(PoolHandle pool_handle,
                                      void (*cb)(CallbackId cb_id, ErrorCode err, const char *response),
                                      CallbackId cb_id);

ErrorCode indy_vdr_pool_refresh(PoolHandle pool_handle,
                                void (*cb)(CallbackId cb_id, ErrorCode err),
                                CallbackId cb_id);

ErrorCode indy_vdr_pool_submit_action(PoolHandle pool_handle,
                                      RequestHandle request_handle,
                                      FfiStr nodes,
                                      int32_t timeout,
                                      void (*cb)(CallbackId cb_id, ErrorCode err, const char *response),
                                      CallbackId cb_id);

ErrorCode indy_vdr_pool_submit_request(PoolHandle pool_handle,
                                       RequestHandle request_handle,
                                       void (*cb)(CallbackId cb_id, ErrorCode err, const char *response),
                                       CallbackId cb_id);

/**
 *
 */
ErrorCode indy_vdr_prepare_txn_author_agreement_acceptance(FfiStr text,
                                                           FfiStr version,
                                                           FfiStr taa_digest,
                                                           FfiStr acc_mech_type,
                                                           uint64_t time,
                                                           const char **output_p);

/**
 * Deallocate a Request instance.
 *
 * @param request_handle handle for the Request instance
 */
ErrorCode indy_vdr_request_free(RequestHandle request_handle);

/**
 * Fetch the body of a request instance.
 *
 * @param request_handle handle for the Request instance
 * @param body_p assigned a pointer to the request body JSON on success
 */
ErrorCode indy_vdr_request_get_body(RequestHandle request_handle, const char **body_p);

ErrorCode indy_vdr_request_get_signature_input(RequestHandle request_handle, const char **input_p);

ErrorCode indy_vdr_request_set_endorser(RequestHandle request_handle, FfiStr endorser);

ErrorCode indy_vdr_request_set_multi_signature(RequestHandle request_handle,
                                               FfiStr identifier,
                                               ByteBuffer signature);

ErrorCode indy_vdr_request_set_signature(RequestHandle request_handle, ByteBuffer signature);

ErrorCode indy_vdr_request_set_txn_author_agreement_acceptance(RequestHandle request_handle,
                                                               FfiStr acceptance);

ErrorCode indy_vdr_set_config(FfiStr config);

ErrorCode indy_vdr_set_default_logger(void);

ErrorCode indy_vdr_set_protocol_version(int64_t version);

ErrorCode indy_vdr_set_socks_proxy(FfiStr socks_proxy);

char *indy_vdr_version(void);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
