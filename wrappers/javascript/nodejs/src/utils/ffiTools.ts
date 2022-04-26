import { Callback } from 'ffi-napi'
import { performance } from 'perf_hooks'
import { alloc } from 'ref-napi'

import {
  ByteBuffer,
  ByteBufferArray,
  FFI_CALLBACK_ID,
  FFI_ERROR_CODE,
  FFI_HANDLE,
  FFI_INT32,
  FFI_STRING,
  FFI_VOID,
} from '../ffi'

/*
 * Handle can have at most 48bits / 6 bytes - because:
 * All numbers in Javascript are 64-bit floating point numbers, which is sufficient to represent any 48-bit integer.
 * Source: https://stackoverflow.com/questions/2575523/48-bit-bitwise-operations-in-javascript
 */
export const allocateHandleBuffer = (): Buffer => alloc(FFI_HANDLE)

export const allocateStringBuffer = (): Buffer => alloc(FFI_STRING)

export const uint8ArrayToByteBuffer = (typedArray: Buffer) => {
  const len = typedArray.length
  const data = new ByteBufferArray(typedArray)

  return ByteBuffer({
    len,
    data,
  })
}

export type NativeCallback = (id: number, errorCode: number) => void
export const toNativeCallback = (cb: NativeCallback) => {
  const callback = Callback(FFI_VOID, [FFI_CALLBACK_ID, FFI_ERROR_CODE], cb)
  const id = setTimeout(() => callback, 1000000)
  return { callback, id }
}

export type NativeCallbackWithResponse = (id: number, errorCode: number, response: string) => void
export const toNativeCallbackWithResponse = (cb: NativeCallbackWithResponse) => {
  const callback = Callback(FFI_VOID, [FFI_CALLBACK_ID, FFI_ERROR_CODE, FFI_STRING], cb)
  const id = setTimeout(() => callback, 1000000)
  return { callback, id }
}
