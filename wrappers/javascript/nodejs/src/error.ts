import type { IndyVdrErrorObject } from 'indy-vdr-shared'

import { IndyVdrError } from 'indy-vdr-shared'

import { nativeIndyVdr } from './lib'
import { allocateStringBuffer } from './utils'

export const handleError = (code: number) => {
  if (code == 0) return

  const nativeError = allocateStringBuffer()
  nativeIndyVdr.indy_vdr_get_current_error(nativeError)

  // TODO
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-argument
  const indyVdrErrorObject: IndyVdrErrorObject = JSON.parse(nativeError.deref())

  throw new IndyVdrError(indyVdrErrorObject)
}
