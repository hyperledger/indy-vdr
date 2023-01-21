import type { IndyVdrPool } from '@hyperledger/indy-vdr-nodejs'

import { NymRequest } from '@hyperledger/indy-vdr-nodejs'

import { DID, setupPool } from './utils'

describe('NymRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new NymRequest({
      dest: DID,
      submitterDid: DID,
    })

    await expect(pool.submitRequest(request)).rejects.toThrowError('MissingSignature()')
  })
})
