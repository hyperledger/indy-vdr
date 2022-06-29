import type { IndyVdrPool } from 'indy-vdr-shared'

import { GetValidatorInfoRequest } from 'indy-vdr-shared'

import { DID, setupPool } from './utils'

describe('GetValidatorInfoRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetValidatorInfoRequest({ submitterDid: DID })

    await expect(pool.submitRequest({ requestHandle: request.handle })).resolves.toMatchObject({})
  })
})
