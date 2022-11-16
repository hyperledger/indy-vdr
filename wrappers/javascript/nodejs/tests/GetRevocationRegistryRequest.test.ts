import type { IndyVdrPool } from 'indy-vdr-shared'

import { GetRevocationRegistryRequest } from 'indy-vdr-shared'

import { REVOC_REG_DEF_ID, setupPool } from './utils'

describe('GetRevocationRegistryRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetRevocationRegistryRequest({ revocationRegistryId: REVOC_REG_DEF_ID, timestamp: new Date() })

    await expect(pool.submitRequest(request)).resolves.toMatchObject({ op: 'REPLY' })
  })
})
