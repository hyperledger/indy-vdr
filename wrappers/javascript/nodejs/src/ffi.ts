import { refType, types } from 'ref-napi'

// FFI Type Strings
export const FFI_HANDLE = 'int64'
export const FFI_HANDLE_POINTER = 'int64*'
export const FFI_ERROR_CODE = 'int'
export const FFI_REQUEST_HANDLE = FFI_HANDLE
export const FFI_POOL_HANDLE = FFI_HANDLE
export const FFI_CALLBACK_ID = 'int64'
export const FFI_STRING = 'string'
export const FFI_STRING_POINTER = refType(types.CString)
export const FFI_CALLBACK_PTR = 'pointer'
export const FFI_INT64 = 'int64'
export const FFI_INT32 = 'int32'
export const FFI_UINT64 = 'uint64'

export const nativeBindings = {
  // first element is method return type, second element is list of method argument types
  indy_vdr_set_config: [FFI_ERROR_CODE, [FFI_STRING]],
  indy_vdr_set_default_logger: [FFI_ERROR_CODE, []],
  indy_vdr_set_protocol_version: [FFI_ERROR_CODE, [FFI_INT64]],
  indy_vdr_set_socks_proxy: [FFI_ERROR_CODE, [FFI_STRING]],
  indy_vdr_version: [FFI_STRING, []],
  // TODO: output pointer for last param
  indy_vdr_get_current_error: [FFI_ERROR_CODE, [FFI_STRING_POINTER]],

  // requests
  indy_vdr_build_acceptance_mechanisms_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE],
  ],
  indy_vdr_build_attrib_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE],
  ],
  indy_vdr_build_cred_def_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE]],
  indy_vdr_build_custom_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_REQUEST_HANDLE]],
  indy_vdr_build_disable_all_txn_author_agreements_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_REQUEST_HANDLE]],
  indy_vdr_build_get_acceptance_mechanisms_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_INT64, FFI_STRING, FFI_REQUEST_HANDLE],
  ],
  indy_vdr_build_get_attrib_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE],
  ],
  indy_vdr_build_get_cred_def_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE]],
  indy_vdr_build_get_nym_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE]],
  indy_vdr_build_get_revoc_reg_def_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE]],
  indy_vdr_build_get_revoc_reg_delta_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_INT64, FFI_INT64, FFI_REQUEST_HANDLE],
  ],
  indy_vdr_build_get_revoc_reg_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_STRING, FFI_INT64, FFI_REQUEST_HANDLE]],
  // indy_vdr_build_get_rich_schema_object_by_id_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE]],
  //indy_vdr_build_get_rich_schema_object_by_metadata_request: [
  //   FFI_ERROR_CODE,
  //   [FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE],
  // ],
  // indy_vdr_build_rich_schema_request: [
  //   FFI_ERROR_CODE,
  //   [FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE],
  // ],
  indy_vdr_build_get_schema_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE]],
  indy_vdr_build_txn_author_agreement_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE]],
  indy_vdr_build_get_txn_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_INT32, FFI_INT32, FFI_REQUEST_HANDLE]],
  indy_vdr_build_get_validator_info_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_REQUEST_HANDLE]],
  indy_vdr_build_nym_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE],
  ],
  indy_vdr_build_revoc_reg_def_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE]],
  indy_vdr_build_revoc_reg_entry_request: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE],
  ],
  indy_vdr_build_schema_request: [FFI_ERROR_CODE, [FFI_STRING, FFI_STRING, FFI_REQUEST_HANDLE]],

  // pool
  // TODO: poolhandle pointer
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

  // TODO
  // TODO: output pointer for last param
  indy_vdr_prepare_txn_author_agreement_acceptance: [
    FFI_ERROR_CODE,
    [FFI_STRING, FFI_STRING, FFI_STRING, FFI_STRING, FFI_UINT64, FFI_STRING],
  ],
  indy_vdr_request_free: [FFI_ERROR_CODE, [FFI_REQUEST_HANDLE]],
  // TODO: output pointer for last param
  indy_vdr_request_get_body: [FFI_ERROR_CODE, [FFI_REQUEST_HANDLE, FFI_STRING]],
  // TODO: output pointer for last param
  indy_vdr_request_get_signature_input: [FFI_ERROR_CODE, [FFI_REQUEST_HANDLE, FFI_STRING]],
  indy_vdr_request_set_endorser: [FFI_ERROR_CODE, [FFI_REQUEST_HANDLE, FFI_STRING]],
  // TODO: FFI STRUCT BYTEBUFFER
  indy_vdr_request_set_multi_signature: [FFI_ERROR_CODE, [FFI_REQUEST_HANDLE, FFI_STRING]],
} as const

// We need a mapping from string type value => type (property 'string' maps to type string)
interface StringTypeMapping {
  string: string
  number: number
  int: number
  int64: number
  int32: number
  uint64: number
  pointer: Buffer
  'CString*': Buffer
  'int64*': Buffer
}

// Needed so TS stops complaining about index signatures...
type ShapeOf<T> = {
  [Property in keyof T]: T[Property]
}
type StringTypeArrayToTypes<List extends Array<keyof StringTypeMapping>> = {
  [Item in keyof List]: List[Item] extends keyof StringTypeMapping ? StringTypeMapping[List[Item]] : never
}

type TypedMethods<Base extends { [method: string | number | symbol]: [any, any[]] }> = {
  [Property in keyof Base]: (
    ...args: StringTypeArrayToTypes<Base[Property][1]> extends any[] ? StringTypeArrayToTypes<Base[Property][1]> : []
  ) => StringTypeMapping[Base[Property][0]]
}

type Mutable<T> = {
  -readonly [K in keyof T]: Mutable<T[K]>
}

export type NativeMethods = TypedMethods<ShapeOf<Mutable<typeof nativeBindings>>>
