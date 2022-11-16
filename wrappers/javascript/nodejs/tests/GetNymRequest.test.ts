import type { IndyVdrPool } from 'indy-vdr-nodejs'

import { DID, setupPool } from './utils'

import { GetNymRequest } from 'indy-vdr-nodejs'

describe('GetNymRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetNymRequest({ dest: DID })

    await expect(pool.submitRequest(request)).resolves.toMatchObject({
      op: 'REPLY',
    })
  })
})
