import { alloc } from 'ref-napi'

import { FFI_HANDLE, FFI_STRING } from '../ffi'

/*
 * Handle can have at most 48bits / 6 bytes - because:
 * All numbers in Javascript are 64-bit floating point numbers, which is sufficient to represent any 48-bit integer.
 * Source: https://stackoverflow.com/questions/2575523/48-bit-bitwise-operations-in-javascript
 */
export const allocateHandleBuffer = (): Buffer => alloc(FFI_HANDLE)

export const allocateStringBuffer = (): Buffer => alloc(FFI_STRING)
