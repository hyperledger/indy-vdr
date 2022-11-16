import type { IndyVdrPool } from 'indy-vdr-shared'

import { RevocationRegistryEntryRequest } from 'indy-vdr-shared'

import { DID, REVOC_REG_DEF_ID, setupPool } from './utils'

describe('RevocationRegistryEntryRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new RevocationRegistryEntryRequest({
      revocationRegistryDefinitionType: 'CL_ACCUM',
      revocationRegistryDefinitionId: REVOC_REG_DEF_ID,
      revocationRegistryEntry: { value: { foo: 'bar', accum: 'foo' }, ver: '1.0' },
      submitterDid: DID,
    })

    await expect(pool.submitRequest(request)).rejects.toThrowError('MissingSignature()')
  })
})
