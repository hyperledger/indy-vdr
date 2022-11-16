import type { IndyVdrPool } from 'indy-vdr-nodejs'

import { DID, setupPool } from './utils'

import { GetValidatorInfoRequest } from 'indy-vdr-nodejs'

describe('GetValidatorInfoRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetValidatorInfoRequest({ submitterDid: DID })

    await expect(pool.submitRequest(request)).resolves.toMatchObject({})
  })
})
