import type { IndyVdrPool } from 'indy-vdr-nodejs'

import { DID, setupPool } from './utils'

import { NymRequest } from 'indy-vdr-nodejs'

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
