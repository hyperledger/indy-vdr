import refArray from 'ref-array-di'
import * as ref from 'ref-napi'
import refStruct from 'ref-struct-di'

const Struct = refStruct(ref)
const Array = refArray(ref)

export const ByteBufferArray = Array(ref.types.uint8)

export const ByteBuffer = Struct({
  len: ref.types.int64,
  data: ByteBufferArray,
})
