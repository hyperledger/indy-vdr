import type { IndyVdrPool } from 'indy-vdr-shared'

import { NymRequest } from 'indy-vdr-shared'

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
