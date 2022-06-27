import type { GetRevocationRegistryDefinitionResponse, IndyVdrPool } from 'indy-vdr-shared'

import { GetRevocationRegistryDefinitionRequest } from 'indy-vdr-shared'

import { REVOC_REG_DEF_ID, setupPool } from './utils'

describe('GetRevocationRegistryDefinitionRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetRevocationRegistryDefinitionRequest({ revocationRegistryId: REVOC_REG_DEF_ID })

    await expect(
      pool.submitRequest<GetRevocationRegistryDefinitionResponse>({ requestHandle: request.handle })
    ).resolves.toMatchObject({ op: 'REPLY' })
  })
})
