import type { GetRevocationRegistryDeltaResponse, IndyVdrPool } from 'indy-vdr-nodejs'

import { REVOC_REG_DEF_ID, setupPool } from './utils'

import { GetRevocationRegistryDeltaRequest } from 'indy-vdr-nodejs'

describe('GetRevocationRegistryDeltaRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetRevocationRegistryDeltaRequest({
      revocationRegistryId: REVOC_REG_DEF_ID,
      toTs: 1,
    })
    const response: GetRevocationRegistryDeltaResponse = await pool.submitRequest(request)

    expect(response).toMatchObject({
      op: 'REPLY',
    })
  })
})
