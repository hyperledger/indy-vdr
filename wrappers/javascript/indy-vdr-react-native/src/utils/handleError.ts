import type { ReturnObject } from './serialize'
import type { IndyVdrErrorObject } from '@hyperledger/indy-vdr-shared'

import { indyVdr, IndyVdrError } from '@hyperledger/indy-vdr-shared'

export const handleError = <T>({ errorCode, value }: ReturnObject<T>): T => {
  if (errorCode !== 0) {
    throw new IndyVdrError(JSON.parse(indyVdr.getCurrentError()) as IndyVdrErrorObject)
  }

  return value as T
}
