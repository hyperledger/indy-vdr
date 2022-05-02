import { Callback } from 'ffi-napi'
import { alloc } from 'ref-napi'

import { ByteBuffer, ByteBufferArray, FFI_CALLBACK_ID, FFI_ERROR_CODE, FFI_HANDLE, FFI_STRING, FFI_VOID } from '../ffi'

/*
 * Handle can have at most 48bits / 6 bytes - because:
 * All numbers in Javascript are 64-bit floating point numbers, which is sufficient to represent any 48-bit integer.
 * Source: https://stackoverflow.com/questions/2575523/48-bit-bitwise-operations-in-javascript
 */
export const allocateHandleBuffer = (): Buffer => alloc(FFI_HANDLE)

export const allocateStringBuffer = (): Buffer => alloc(FFI_STRING)

export const allocateCallbackBuffer = (callback: Buffer) => setTimeout(() => callback, 1000000)

export const deallocateCallbackBuffer = (id: number) => clearTimeout(id as unknown as NodeJS.Timeout)

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
  const nativeCallback = Callback(FFI_VOID, [FFI_CALLBACK_ID, FFI_ERROR_CODE], cb)
  const id = allocateCallbackBuffer(nativeCallback)
  return { nativeCallback, id }
}

export type NativeCallbackWithResponse = (id: number, errorCode: number, response: string) => void
export const toNativeCallbackWithResponse = (cb: NativeCallbackWithResponse) => {
  const nativeCallback = Callback(FFI_VOID, [FFI_CALLBACK_ID, FFI_ERROR_CODE, FFI_STRING], cb)
  const id = allocateCallbackBuffer(nativeCallback)
  return { nativeCallback, id }
}
