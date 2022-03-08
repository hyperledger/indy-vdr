#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define DEFAULT_ACK_TIMEOUT 20

#define DEFAULT_CONN_ACTIVE_TIMEOUT 5

#define DEFAULT_CONN_REQUEST_LIMIT 5

#define DEFAULT_FRESHNESS_TIMEOUT 300

#define DEFAULT_REPLY_TIMEOUT 60

#define DEFAULT_REQUEST_READ_NODES 2

#define MAX_ATTRIBUTES_COUNT 125

enum ErrorCode {
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
typedef uintptr_t ErrorCode;
typedef uintptr_t Handle;
typedef uintptr_t CallbackId;
typedef const char *FfiStr;


ErrorCode indy_vdr_build_acceptance_mechanisms_request(FfiStr submitter_did,
                                                       FfiStr aml,
                                                       FfiStr version,
                                                       FfiStr aml_context,
                                                       Handle *handle_p);

ErrorCode indy_vdr_build_cred_def_request(FfiStr submitter_did,
                                          FfiStr cred_def,
                                          Handle *handle_p);

ErrorCode indy_vdr_build_custom_request(FfiStr request_json, Handle *handle_p);

ErrorCode indy_vdr_build_disable_all_txn_author_agreements_request(FfiStr submitter_did,
                                                                   Handle *handle_p);

ErrorCode indy_vdr_build_get_acceptance_mechanisms_request(FfiStr submitter_did,
                                                           int64_t timestamp,
                                                           FfiStr version,
                                                           Handle *handle_p);

ErrorCode indy_vdr_build_get_cred_def_request(FfiStr submitter_did,
                                              FfiStr cred_def_id,
                                              Handle *handle_p);

ErrorCode indy_vdr_build_get_nym_request(FfiStr submitter_did, FfiStr dest, int32_t seq_no, int64_t timestamp, Handle *handle_p);

ErrorCode indy_vdr_build_get_revoc_reg_def_request(FfiStr submitter_did,
                                                   FfiStr revoc_reg_id,
                                                   Handle *handle_p);

ErrorCode indy_vdr_build_get_revoc_reg_delta_request(FfiStr submitter_did,
                                                     FfiStr revoc_reg_id,
                                                     int64_t from_ts,
                                                     int64_t to_ts,
                                                     Handle *handle_p);

ErrorCode indy_vdr_build_get_revoc_reg_request(FfiStr submitter_did,
                                               FfiStr revoc_reg_id,
                                               int64_t timestamp,
                                               Handle *handle_p);

ErrorCode indy_vdr_build_get_rich_schema_object_by_id_request(FfiStr submitter_did,
                                                              FfiStr rs_id,
                                                              Handle *handle_p);

ErrorCode indy_vdr_build_get_rich_schema_object_by_metadata_request(FfiStr submitter_did,
                                                                    FfiStr rs_type,
                                                                    FfiStr rs_name,
                                                                    FfiStr rs_version,
                                                                    Handle *handle_p);

ErrorCode indy_vdr_build_get_schema_request(FfiStr submitter_did,
                                            FfiStr schema_id,
                                            Handle *handle_p);

ErrorCode indy_vdr_build_get_txn_author_agreement_request(FfiStr submitter_did,
                                                          FfiStr data,
                                                          Handle *handle_p);

ErrorCode indy_vdr_build_get_txn_request(FfiStr submitter_did,
                                         int32_t ledger_type,
                                         int32_t seq_no,
                                         Handle *handle_p);

ErrorCode indy_vdr_build_get_validator_info_request(FfiStr submitter_did, Handle *handle_p);

ErrorCode indy_vdr_build_nym_request(FfiStr submitter_did,
                                     FfiStr dest,
                                     FfiStr verkey,
                                     FfiStr alias,
                                     FfiStr role,
                                     FfiStr diddoc_content,
                                     Handle *handle_p);

ErrorCode indy_vdr_build_revoc_reg_def_request(FfiStr submitter_did,
                                               FfiStr revoc_reg_def,
                                               Handle *handle_p);

ErrorCode indy_vdr_build_revoc_reg_entry_request(FfiStr submitter_did,
                                                 FfiStr revoc_reg_def_id,
                                                 FfiStr revoc_reg_def_type,
                                                 FfiStr revoc_reg_entry,
                                                 Handle *handle_p);

ErrorCode indy_vdr_build_rich_schema_request(FfiStr submitter_did,
                                             FfiStr rs_id,
                                             FfiStr rs_content,
                                             FfiStr rs_name,
                                             FfiStr rs_version,
                                             FfiStr rs_type,
                                             FfiStr ver,
                                             Handle *handle_p);

ErrorCode indy_vdr_build_schema_request(FfiStr submitter_did, FfiStr schema, Handle *handle_p);

ErrorCode indy_vdr_build_txn_author_agreement_request(FfiStr submitter_did,
                                                      FfiStr text,
                                                      FfiStr version,
                                                      int64_t ratification_ts,
                                                      int64_t retirement_ts,
                                                      Handle *handle_p);

ErrorCode indy_vdr_get_current_error(const char **error_json_p);

ErrorCode indy_vdr_pool_close(Handle pool_handle);

ErrorCode indy_vdr_pool_create(FfiStr params, Handle *handle_p);

ErrorCode indy_vdr_pool_get_status(Handle pool_handle,
                                   void (*cb)(CallbackId cb_id, ErrorCode err, const char *response),
                                   CallbackId cb_id);

ErrorCode indy_vdr_pool_get_transactions(Handle pool_handle,
                                         void (*cb)(CallbackId cb_id, ErrorCode err, const char *response),
                                         CallbackId cb_id);

ErrorCode indy_vdr_pool_get_verifiers(Handle pool_handle,
                                      void (*cb)(CallbackId cb_id, ErrorCode err, const char *response),
                                      CallbackId cb_id);

ErrorCode indy_vdr_pool_refresh(Handle pool_handle,
                                void (*cb)(CallbackId cb_id, ErrorCode err),
                                CallbackId cb_id);

ErrorCode indy_vdr_pool_submit_action(Handle pool_handle,
                                      Handle request_handle,
                                      FfiStr nodes,
                                      int32_t timeout,
                                      void (*cb)(CallbackId cb_id, ErrorCode err, const char *response),
                                      CallbackId cb_id);

ErrorCode indy_vdr_pool_submit_request(Handle pool_handle,
                                       Handle request_handle,
                                       void (*cb)(CallbackId cb_id, ErrorCode err, const char *response),
                                       CallbackId cb_id);

ErrorCode indy_vdr_prepare_txn_author_agreement_acceptance(FfiStr text,
                                                           FfiStr version,
                                                           FfiStr taa_digest,
                                                           FfiStr acc_mech_type,
                                                           uint64_t time,
                                                           const char **output_p);

ErrorCode indy_vdr_request_free(Handle request_handle);

ErrorCode indy_vdr_request_get_body(Handle request_handle, const char **body_p);

ErrorCode indy_vdr_request_get_signature_input(Handle request_handle, const char **input_p);

ErrorCode indy_vdr_request_set_endorser(Handle request_handle, FfiStr endorser);

ErrorCode indy_vdr_request_set_signature(Handle request_handle,
                                         const uint8_t *signature,
                                         uintptr_t signature_len);

ErrorCode indy_vdr_request_set_txn_author_agreement_acceptance(Handle request_handle,
                                                               FfiStr acceptance);

ErrorCode indy_vdr_set_config(FfiStr config);

ErrorCode indy_vdr_set_default_logger(void);

ErrorCode indy_vdr_set_protocol_version(uintptr_t version);

char *indy_vdr_version(void);

ErrorCode indy_vdr_set_socks_proxy(FfiStr socks_proxy);