import type { IndyVdrPool } from '@hyperledger/indy-vdr-nodejs'

import { DID, setupPool } from './utils'

import { AcceptanceMechanismsRequest } from '@hyperledger/indy-vdr-nodejs'

describe('AcceptanceMechanismsRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new AcceptanceMechanismsRequest({
      aml: { 'acceptance mechanism label 1': { filed: 'value' } },
      submitterDid: DID,
      version: '1.0.0',
    })

    await expect(pool.submitRequest(request)).rejects.toThrowError('MissingSignature()')
  })
})
