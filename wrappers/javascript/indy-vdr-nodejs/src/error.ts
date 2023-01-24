import type { IndyVdrErrorObject } from '@hyperledger/indy-vdr-shared'

import { IndyVdrError } from '@hyperledger/indy-vdr-shared'

import { allocateString } from './ffi'
import { nativeIndyVdr } from './library'

export const handleError = (code: number) => {
  if (code === 0) return

  const nativeError = allocateString()
  nativeIndyVdr.indy_vdr_get_current_error(nativeError)

  const indyVdrErrorObject = JSON.parse(nativeError.deref() as string) as IndyVdrErrorObject

  throw new IndyVdrError(indyVdrErrorObject)
}
