import type { IndyVdrPool } from 'indy-vdr-shared'

import { RevocationRegistryDefinitionRequest } from 'indy-vdr-shared'

import { CRED_DEF_ID, DID, setupPool } from './utils'

describe('RevocationRegistryDefinitionRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new RevocationRegistryDefinitionRequest({
      submitterDid: DID,
      revocationRegistryDefinitionV1: {
        ver: '1.0',
        credDefId: CRED_DEF_ID,
        id: 'TODO',
        revocDefType: 'CL_ACCUM',
        tag: 'TODO',
        value: {
          issuanceType: 'ISSUANCE_BY_DEFAULT',
          maxCredNum: 0,
          publicKeys: { accumKey: 'TODO' },
          tailsHash: 'TODO',
          tailsLocation: 'TODO',
        },
      },
    })

    await expect(pool.submitRequest({ requestHandle: request.handle })).rejects.toThrowError('MissingSignature()')
  })
})
