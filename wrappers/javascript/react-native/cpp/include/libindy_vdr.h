#pragma once

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

/**
 * `FfiStr<'a>` is a safe (`#[repr(transparent)]`) wrapper around a
 * nul-terminated `*const c_char` (e.g. a C string). Conceptually, it is
 * similar to [`std::ffi::CStr`], except that it may be used in the signatures
 * of extern "C" functions.
 *
 * Functions accepting strings should use this instead of accepting a C string
 * directly. This allows us to write those functions using safe code without
 * allowing safe Rust to cause memory unsafety.
 *
 * A single function for constructing these from Rust ([`FfiStr::from_raw`])
 * has been provided. Most of the time, this should not be necessary, and users
 * should accept `FfiStr` in the parameter list directly.
 *
 * ## Caveats
 *
 * An effort has been made to make this struct hard to misuse, however it is
 * still possible, if the `'static` lifetime is manually specified in the
 * struct. E.g.
 *
 * ```rust,no_run
 * # use ffi_support::FfiStr;
 * // NEVER DO THIS
 * #[no_mangle]
 * extern "C" fn never_do_this(s: FfiStr<'static>) {
 *     // save `s` somewhere, and access it after this
 *     // function returns.
 * }
 * ```
 *
 * Instead, one of the following patterns should be used:
 *
 * ```
 * # use ffi_support::FfiStr;
 * #[no_mangle]
 * extern "C" fn valid_use_1(s: FfiStr<'_>) {
 *     // Use of `s` after this function returns is impossible
 * }
 * // Alternative:
 * #[no_mangle]
 * extern "C" fn valid_use_2(s: FfiStr) {
 *     // Use of `s` after this function returns is impossible
 * }
 * ```
 */
typedef const char *FfiStr;

typedef int64_t RequestHandle;

typedef int64_t PoolHandle;

typedef int64_t CallbackId;

/**
 * ByteBuffer is a struct that represents an array of bytes to be sent over the FFI boundaries.
 * There are several cases when you might want to use this, but the primary one for us
 * is for returning protobuf-encoded data to Swift and Java. The type is currently rather
 * limited (implementing almost no functionality), however in the future it may be
 * more expanded.
 *
 * ## Caveats
 *
 * Note that the order of the fields is `len` (an i64) then `data` (a `*mut u8`), getting
 * this wrong on the other side of the FFI will cause memory corruption and crashes.
 * `i64` is used for the length instead of `u64` and `usize` because JNA has interop
 * issues with both these types.
 *
 * ### `Drop` is not implemented
 *
 * ByteBuffer does not implement Drop. This is intentional. Memory passed into it will
 * be leaked if it is not explicitly destroyed by calling [`ByteBuffer::destroy`], or
 * [`ByteBuffer::destroy_into_vec`]. This is for two reasons:
 *
 * 1. In the future, we may allow it to be used for data that is not managed by
 *    the Rust allocator\*, and `ByteBuffer` assuming it's okay to automatically
 *    deallocate this data with the Rust allocator.
 *
 * 2. Automatically running destructors in unsafe code is a
 *    [frequent footgun](https://without.boats/blog/two-memory-bugs-from-ringbahn/)
 *    (among many similar issues across many crates).
 *
 * Note that calling `destroy` manually is often not needed, as usually you should
 * be passing these to the function defined by [`define_bytebuffer_destructor!`] from
 * the other side of the FFI.
 *
 * Because this type is essentially *only* useful in unsafe or FFI code (and because
 * the most common usage pattern does not require manually managing the memory), it
 * does not implement `Drop`.
 *
 * \* Note: in the case of multiple Rust shared libraries loaded at the same time,
 * there may be multiple instances of "the Rust allocator" (one per shared library),
 * in which case we're referring to whichever instance is active for the code using
 * the `ByteBuffer`. Note that this doesn't occur on all platforms or build
 * configurations, but treating allocators in different shared libraries as fully
 * independent is always safe.
 *
 * ## Layout/fields
 *
 * This struct's field are not `pub` (mostly so that we can soundly implement `Send`, but also so
 * that we can verify rust users are constructing them appropriately), the fields, their types, and
 * their order are *very much* a part of the public API of this type. Consumers on the other side
 * of the FFI will need to know its layout.
 *
 * If this were a C struct, it would look like
 *
 * ```c,no_run
 * struct ByteBuffer {
 *     // Note: This should never be negative, but values above
 *     // INT64_MAX / i64::MAX are not allowed.
 *     int64_t len;
 *     // Note: nullable!
 *     uint8_t *data;
 * };
 * ```
 *
 * In rust, there are two fields, in this order: `len: i64`, and `data: *mut u8`.
 *
 * For clarity, the fact that the data pointer is nullable means that `Option<ByteBuffer>` is not
 * the same size as ByteBuffer, and additionally is not FFI-safe (the latter point is not
 * currently guaranteed anyway as of the time of writing this comment).
 *
 * ### Description of fields
 *
 * `data` is a pointer to an array of `len` bytes. Note that data can be a null pointer and therefore
 * should be checked.
 *
 * The bytes array is allocated on the heap and must be freed on it as well. Critically, if there
 * are multiple rust shared libraries using being used in the same application, it *must be freed
 * on the same heap that allocated it*, or you will corrupt both heaps.
 *
 * Typically, this object is managed on the other side of the FFI (on the "FFI consumer"), which
 * means you must expose a function to release the resources of `data` which can be done easily
 * using the [`define_bytebuffer_destructor!`] macro provided by this crate.
 */
typedef struct ByteBuffer {
  int64_t len;
  uint8_t *data;
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

#if (defined(DEFINE_RICH_SCHEMA) || defined(DEFINE_TEST))
ErrorCode indy_vdr_build_get_rich_schema_object_by_id_request(FfiStr submitter_did,
                                                              FfiStr rs_id,
                                                              RequestHandle *handle_p);
#endif

#if (defined(DEFINE_RICH_SCHEMA) || defined(DEFINE_TEST))
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

#if (defined(DEFINE_RICH_SCHEMA) || defined(DEFINE_TEST))
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
                                               struct ByteBuffer signature);

ErrorCode indy_vdr_request_set_signature(RequestHandle request_handle, struct ByteBuffer signature);

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
