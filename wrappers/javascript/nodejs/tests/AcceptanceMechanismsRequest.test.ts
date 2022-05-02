import type { IndyVdrPool } from 'indy-vdr-shared'

import { AcceptanceMechanismsRequest } from 'indy-vdr-shared'

import { DID, setupPool } from './utils'

describe('AcceptanceMechanismsRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new AcceptanceMechanismsRequest({
      aml: { 'acceptance mechanism label 1': { filed: 'value' } },
      submitterDid: DID,
      version: '1.0.0',
    })

    await expect(pool.submitRequest({ requestHandle: request.handle })).rejects.toThrowError('MissingSignature()')
  })
})
