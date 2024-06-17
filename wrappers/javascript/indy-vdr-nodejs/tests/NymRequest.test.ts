import { NymRequest } from '@hyperledger/indy-vdr-nodejs'
import { describe, expect, test } from 'vitest'
import { DID, setupPool } from './utils'

describe('NymRequest', () => {
  const pool = setupPool()

  test('Submit request', async () => {
    const request = new NymRequest({
      dest: DID,
      submitterDid: DID,
    })

    await expect(pool.submitRequest(request)).rejects.toThrowError(
      'MissingSignature()',
    )
  })
})
