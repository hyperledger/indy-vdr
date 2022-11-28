import type { GetRevocationRegistryResponse, IndyVdrPool } from 'indy-vdr-nodejs'

import { REVOC_REG_DEF_ID, setupPool } from './utils'

import { GetRevocationRegistryRequest } from 'indy-vdr-nodejs'

describe('GetRevocationRegistryRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetRevocationRegistryRequest({ revocationRegistryId: REVOC_REG_DEF_ID, timestamp: new Date() })
    const response: GetRevocationRegistryResponse = await pool.submitRequest(request)

    expect(response).toMatchObject({ op: 'REPLY' })
  })
})
