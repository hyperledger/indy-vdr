import { describe, expect, test } from 'vitest'
import { DID, setupPool } from './utils'

import { AcceptanceMechanismsRequest } from '@hyperledger/indy-vdr-nodejs'
console.log('hell')
describe('AcceptanceMechanismsRequest', () => {
  const pool = setupPool()

  test('Submit request', async () => {
    const request = new AcceptanceMechanismsRequest({
      aml: { 'acceptance mechanism label 1': { filed: 'value' } },
      submitterDid: DID,
      version: '1.0.0',
    })

    await expect(pool.submitRequest(request)).rejects.toThrowError(
      'MissingSignature()',
    )
  })
})
