import type { IndyVdrPool, GetRevocationRegistryDefinitionResponse } from '@hyperledger/indy-vdr-nodejs'

import { REVOC_REG_DEF_ID, setupPool } from './utils'

import { GetRevocationRegistryDefinitionRequest } from '@hyperledger/indy-vdr-nodejs'

describe('GetRevocationRegistryDefinitionRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetRevocationRegistryDefinitionRequest({ revocationRegistryId: REVOC_REG_DEF_ID })
    const response: GetRevocationRegistryDefinitionResponse = await pool.submitRequest(request)

    expect(response).toMatchObject({
      op: 'REPLY',
    })
  })
})
