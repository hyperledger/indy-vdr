import type { IndyVdrPool } from '@hyperledger/indy-vdr-nodejs'

import { DID, REVOC_REG_DEF_ID, setupPool } from './utils'

import { RevocationRegistryEntryRequest } from '@hyperledger/indy-vdr-nodejs'

describe('RevocationRegistryEntryRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new RevocationRegistryEntryRequest({
      revocationRegistryDefinitionType: 'CL_ACCUM',
      revocationRegistryDefinitionId: REVOC_REG_DEF_ID,
      revocationRegistryEntry: { value: { accum: 'foo' }, ver: '1.0' },
      submitterDid: DID,
    })

    await expect(pool.submitRequest(request)).rejects.toThrowError('MissingSignature()')
  })
})
