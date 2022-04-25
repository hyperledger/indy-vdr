import type { IndyVdrErrorObject } from 'indy-vdr-shared'

import { IndyVdrError } from 'indy-vdr-shared'

import { indyVdr } from './lib'
import { allocateStringBuffer } from './utils'

export const handleError = () => {
  const nativeError = allocateStringBuffer()
  indyVdr.indy_vdr_get_current_error(nativeError)

  // TODO
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-argument
  const indyVdrErrorObject: IndyVdrErrorObject = JSON.parse(nativeError.deref())

  throw new IndyVdrError(indyVdrErrorObject)
}
