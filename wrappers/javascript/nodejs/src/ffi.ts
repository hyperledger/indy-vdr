import { default as array } from 'ref-array-di'
import * as ref from 'ref-napi'
import { default as struct } from 'ref-struct-di'

const Struct = struct(ref)
const Array = array(ref)

export const ByteBufferArray = Array(ref.types.uint8)

export const ByteBuffer = Struct({
  len: ref.types.int64,
  data: ByteBufferArray,
})

// FFI Type Strings
export const FFI_ERROR_CODE = 'int'

export const FFI_HANDLE = 'int64'
export const FFI_HANDLE_POINTER = 'int64*'

export const FFI_REQUEST_HANDLE = FFI_HANDLE
export const FFI_REQUEST_HANDLE_POINTER = FFI_HANDLE_POINTER
export const FFI_POOL_HANDLE = FFI_HANDLE

export const FFI_CALLBACK_ID = 'int64'
export const FFI_CALLBACK_PTR = 'pointer'

export const FFI_STRING = 'string'
export const FFI_STRING_POINTER = 'char*'

export const FFI_INT64 = 'int64'
export const FFI_INT32 = 'int32'
export const FFI_UINT64 = 'uint64'
export const FFI_VOID = 'void'

export const nativeBindings = {
  // first element is method return type, second element is list of method argument types
  indy_vdr_set_config: [FFI_ERROR_CODE, [FFI_STRING]],
  indy_vdr_set_default_logger: [FFI_ERROR_CODE, []],
  indy_vdr_set_protocol_version: [FFI_ERROR_CODE, [FFI_INT64]],
  indy_vdr_set_socks_proxy: [FFI_ERROR_CODE, [FFI_STRING]],
  indy_vdr_version: [FFI_STRING, []],
  indy_vdr_get_current_error: [FFI_ERROR_CODE, [FFI_STRING_POINTER]],

  // requests
  indy_vdr_build_acceptance_mechanisms_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE_POINTER],
  ],
  indy_vdr_build_attrib_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE_POINTER],
  ],
  indy_vdr_build_cred_def_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE_POINTER]],
  indy_vdr_build_custom_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_REQUEST_HANDLE_POINTER]],
  indy_vdr_build_disable_all_txn_author_agreements_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_REQUEST_HANDLE_POINTER]],
  indy_vdr_build_get_acceptance_mechanisms_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_INT64, FFI_STRING, FFI_REQUEST_HANDLE_POINTER],
  ],
  indy_vdr_build_get_attrib_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE_POINTER],
  ],
  indy_vdr_build_get_cred_def_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE_POINTER]],
  indy_vdr_build_get_nym_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE_POINTER]],
  indy_vdr_build_get_revoc_reg_def_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE_POINTER]],
  indy_vdr_build_get_revoc_reg_delta_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_INT64, FFI_INT64, FFI_REQUEST_HANDLE_POINTER],
  ],
  indy_vdr_build_get_revoc_reg_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_INT64, FFI_REQUEST_HANDLE_POINTER],
  ],
  indy_vdr_build_get_rich_schema_object_by_id_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE_POINTER],
  ],
  indy_vdr_build_get_rich_schema_object_by_metadata_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE_POINTER],
  ],
  indy_vdr_build_rich_schema_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE_POINTER],
  ],
  indy_vdr_build_get_schema_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE_POINTER]],
  indy_vdr_build_txn_author_agreement_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_STRING, FFI_INT64, FFI_INT64, FFI_REQUEST_HANDLE_POINTER],
  ],
  indy_vdr_build_get_txn_author_agreement_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE_POINTER],
  ],
  indy_vdr_build_get_txn_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_INT32, FFI_INT32, FFI_REQUEST_HANDLE_POINTER]],
  indy_vdr_build_get_validator_info_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_REQUEST_HANDLE_POINTER]],
  indy_vdr_build_nym_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE_POINTER],
  ],
  indy_vdr_build_revoc_reg_def_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE_POINTER]],
  indy_vdr_build_revoc_reg_entry_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE_POINTER],
  ],
  indy_vdr_build_schema_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE_POINTER]],

  // pool
  indy_vdr_pool_create: [FFI_ERROR_CODE, [FFI_STRING, FFI_HANDLE_POINTER]],
  indy_vdr_pool_close: [FFI_ERROR_CODE, [FFI_POOL_HANDLE]],
  indy_vdr_pool_refresh: [FFI_ERROR_CODE, [FFI_POOL_HANDLE, FFI_CALLBACK_PTR, FFI_CALLBACK_ID]],
  indy_vdr_pool_submit_action: [
    FFI_ERROR_CODE,
    [FFI_POOL_HANDLE, FFI_REQUEST_HANDLE, FFI_STRING, FFI_INT32, FFI_CALLBACK_PTR, FFI_CALLBACK_ID],
  ],
  indy_vdr_pool_submit_request: [
    FFI_ERROR_CODE,
    [FFI_POOL_HANDLE, FFI_REQUEST_HANDLE, FFI_CALLBACK_PTR, FFI_CALLBACK_ID],
  ],
  indy_vdr_pool_get_status: [FFI_ERROR_CODE, [FFI_POOL_HANDLE, FFI_CALLBACK_PTR, FFI_CALLBACK_ID]],
  indy_vdr_pool_get_transactions: [FFI_ERROR_CODE, [FFI_POOL_HANDLE, FFI_CALLBACK_PTR, FFI_CALLBACK_ID]],
  indy_vdr_pool_get_verifiers: [FFI_ERROR_CODE, [FFI_POOL_HANDLE, FFI_CALLBACK_PTR, FFI_CALLBACK_ID]],

  indy_vdr_prepare_txn_author_agreement_acceptance: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_UINT64, FFI_STRING_POINTER],
  ],
  indy_vdr_request_free: [FFI_ERROR_CODE, [FFI_REQUEST_HANDLE]],
  indy_vdr_request_get_body: [FFI_ERROR_CODE, [FFI_REQUEST_HANDLE, FFI_STRING_POINTER]],
  indy_vdr_request_get_signature_input: [FFI_ERROR_CODE, [FFI_REQUEST_HANDLE, FFI_STRING_POINTER]],
  indy_vdr_request_set_endorser: [FFI_ERROR_CODE, [FFI_REQUEST_HANDLE, FFI_STRING]],
  indy_vdr_request_set_multi_signature: [FFI_ERROR_CODE, [FFI_REQUEST_HANDLE, FFI_STRING, ByteBuffer]],
  indy_vdr_request_set_signature: [FFI_ERROR_CODE, [FFI_REQUEST_HANDLE, ByteBuffer]],
  indy_vdr_request_set_txn_author_agreement_acceptance: [FFI_ERROR_CODE, [FFI_REQUEST_HANDLE, FFI_STRING]],
} as const

// TODO handle optional
export interface NativeMethods {
  indy_vdr_set_config: (arg0: string) => number
  indy_vdr_set_default_logger: () => number
  indy_vdr_set_protocol_version: (arg0: number) => number
  indy_vdr_set_socks_proxy: (arg0: string) => number
  indy_vdr_version: () => string
  indy_vdr_get_current_error: (arg0: Buffer) => number
  indy_vdr_build_acceptance_mechanisms_request: (
    arg0: string,
    arg1: string,
    arg2: string,
    arg3: string,
    requestHandlePtr: Buffer
  ) => number
  indy_vdr_build_attrib_request: (
    arg0: string,
    arg1: string,
    arg2: string,
    arg3: string,
    arg4: string,
    requestHandlePtr: Buffer
  ) => number
  indy_vdr_build_cred_def_request: (arg0: string, arg1: string, requestHandlePtr: Buffer) => number
  indy_vdr_build_custom_request: (arg0: string, requestHandlePtr: Buffer) => number
  indy_vdr_build_disable_all_txn_author_agreements_request: (arg0: string, requestHandlePtr: Buffer) => number
  indy_vdr_build_get_acceptance_mechanisms_request: (
    arg0: string,
    arg1: number,
    arg2: string,
    requestHandlePtr: Buffer
  ) => number
  indy_vdr_build_get_cred_def_request: (arg0: string, arg1: string, requestHandlePtr: Buffer) => number
  indy_vdr_build_get_revoc_reg_def_request: (arg0: string, arg1: string, requestHandlePtr: Buffer) => number
  indy_vdr_build_get_revoc_reg_delta_request: (
    arg0: string,
    arg1: string,
    arg2: number,
    arg3: number,
    requestHandlePtr: Buffer
  ) => number
  indy_vdr_build_get_revoc_reg_request: (arg0: string, arg1: string, arg2: number, requestHandlePtr: Buffer) => number
  indy_vdr_build_get_schema_request: (arg0: string, arg1: string, requestHandlePtr: Buffer) => number
  indy_vdr_build_get_attrib_request: (
    arg0: string,
    arg1: string,
    arg2: string,
    arg3: string,
    arg4: string,
    requestHandlePtr: Buffer
  ) => number
  indy_vdr_build_node_request: (arg0: string, arg1: string, arg2: string, requestHandlePtr: Buffer) => number
  indy_vdr_build_pool_config_request: (arg0: string, arg1: string, requestHandlePtr: Buffer) => number
  indy_vdr_build_revoc_reg_def_request: (arg0: string, arg1: string, requestHandlePtr: Buffer) => number
  indy_vdr_build_revoc_reg_entry_request: (
    arg0: string,
    arg1: string,
    arg2: string,
    arg3: string,
    requestHandlePtr: Buffer
  ) => number
  indy_vdr_build_schema_request: (arg0: string, arg1: string, requestHandlePtr: Buffer) => number
  indy_vdr_build_txn_author_agreement_request: (
    arg0: string,
    arg1: string,
    arg2: string,
    arg3: number,
    arg4: number,
    requestHandlePtr: Buffer
  ) => number
  indy_vdr_build_get_txn_author_agreement_request: (arg0: string, arg1: string, requestHandlePtr: Buffer) => number
  indy_vdr_build_get_nym_request: (arg0: string, arg1: string, requestHandlePtr: Buffer) => number
  indy_vdr_build_get_rich_schema_object_by_id_request: (arg0: string, arg1: string, requestHandlePtr: Buffer) => number
  indy_vdr_build_get_rich_schema_object_by_metadata_request: (
    arg0: string,
    arg1: string,
    arg2: string,
    arg3: string,
    requestHandlePtr: Buffer
  ) => number
  indy_vdr_build_rich_schema_request: (
    arg0: string,
    arg1: string,
    arg2: string,
    arg3: string,
    arg4: string,
    arg5: string,
    arg6: string,
    requestHandlePtr: Buffer
  ) => number
  indy_vdr_build_get_txn_request: (arg0: string, arg1: number, arg2: number, requestHandlePtr: Buffer) => number
  indy_vdr_build_get_validator_info_request: (arg0: string, requestHandlePtr: Buffer) => number
  indy_vdr_build_nym_request: (
    arg0: string,
    arg1: string,
    arg2: string,
    arg3: string,
    arg4: string,
    requestHandlePtr: Buffer
  ) => number
  indy_vdr_pool_create: (arg0: string, poolHandlePtr: Buffer) => number
  indy_vdr_pool_close: (poolHandle: number) => number
  indy_vdr_pool_refresh: (poolHandle: number, callBackPtr: Buffer, callbackId: number) => number
  indy_vdr_pool_submit_action: (
    poolHandle: number,
    requestHandle: number,
    arg0: string,
    arg1: number,
    callBackPtr: Buffer,
    callbackId: number
  ) => number
  indy_vdr_pool_submit_request: (
    poolHandle: number,
    requestHandle: number,
    callBackPtr: Buffer,
    callbackId: number
  ) => number
  indy_vdr_pool_get_status: (poolHandle: number, callBackPtr: Buffer, callbackId: number) => number
  indy_vdr_pool_get_transactions: (poolHandle: number, callBackPtr: Buffer, callbackId: number) => number
  indy_vdr_pool_get_verifiers: (poolHandle: number, callBackPtr: Buffer, callbackId: number) => number
  indy_vdr_prepare_txn_author_agreement_acceptance: (
    arg0: string,
    arg1: string,
    arg2: string,
    arg3: string,
    arg4: number,
    outputPtr: Buffer
  ) => number
  indy_vdr_request_free: (requestHandle: number) => number
  indy_vdr_request_get_body: (requestHanlde: number, outputPtr: Buffer) => number
  indy_vdr_request_get_signature_input: (requestHandle: number, outputPtr: Buffer) => number
  indy_vdr_request_set_endorser: (requestHandle: number, arg0: string) => number
  indy_vdr_request_set_multi_signature: (requestHandle: number, arg0: string, arg1: typeof ByteBuffer) => number
  indy_vdr_request_set_signature: (requestHandle: number, arg0: typeof ByteBuffer) => number
  indy_vdr_request_set_txn_author_agreement_acceptance: (requestHandle: number, arg0: string) => number
}
