import { refType } from 'ref-napi'

// Primitives

export const FFI_UINT8 = 'uint8'
export const FFI_UINT64 = 'uint64'
export const FFI_USIZE = 'size_t'
export const FFI_INT8 = 'int8'
export const FFI_INT32 = 'int32'
export const FFI_INT64 = 'int64'
export const FFI_STRING = 'string'
export const FFI_VOID = 'void'
export const FFI_POINTER = 'pointer'

// Pointers

export const FFI_INT8_PTR = refType(FFI_INT8)
export const FFI_STRING_PTR = refType(FFI_STRING)
export const FFI_INT32_PTR = refType(FFI_INT32)

// Custom

export const FFI_CALLBACK_ID = FFI_INT64
export const FFI_CALLBACK_PTR = FFI_POINTER
export const FFI_ERROR_CODE = FFI_INT64

export const FFI_HANDLE = 'int64'
export const FFI_HANDLE_POINTER = 'int64*'

export const FFI_REQUEST_HANDLE = FFI_HANDLE
export const FFI_REQUEST_HANDLE_POINTER = FFI_HANDLE_POINTER
export const FFI_POOL_HANDLE = FFI_HANDLE
