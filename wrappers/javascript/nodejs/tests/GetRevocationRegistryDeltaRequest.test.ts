import type { GetRevocationRegistryDeltaResponse, IndyVdrPool } from 'indy-vdr-shared'

import { GetRevocationRegistryDeltaRequest } from 'indy-vdr-shared'

import { REVOC_REG_DEF_ID, setupPool } from './utils'

describe('GetRevocationRegistryDeltaRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetRevocationRegistryDeltaRequest({ revocationRegistryId: REVOC_REG_DEF_ID, toTs: 1 })

    await expect(
      pool.submitRequest<GetRevocationRegistryDeltaResponse>({ requestHandle: request.handle })
    ).resolves.toMatchObject({ op: 'REPLY' })
  })
})
