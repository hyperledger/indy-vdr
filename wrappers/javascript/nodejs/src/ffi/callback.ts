import { Callback } from 'ffi-napi'

import { allocateCallback } from './alloc'
import { FFI_VOID, FFI_CALLBACK_ID, FFI_ERROR_CODE, FFI_STRING } from './primitives'

export type NativeCallback = (id: number, errorCode: number) => void
export const toNativeCallback = (cb: NativeCallback) => {
  const nativeCallback = Callback(FFI_VOID, [FFI_CALLBACK_ID, FFI_ERROR_CODE], cb)
  const id = allocateCallback(nativeCallback)
  return { nativeCallback, id }
}

export type NativeCallbackWithResponse = (id: number, errorCode: number, response: string) => void
export const toNativeCallbackWithResponse = (cb: NativeCallbackWithResponse) => {
  const nativeCallback = Callback(FFI_VOID, [FFI_CALLBACK_ID, FFI_ERROR_CODE, FFI_STRING], cb)
  const id = allocateCallback(nativeCallback)
  return { nativeCallback, id }
}
