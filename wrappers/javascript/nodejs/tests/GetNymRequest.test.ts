import type { IndyVdrPool } from 'indy-vdr-shared'

import { GetNymRequest } from 'indy-vdr-shared'

import { DID, setupPool } from './utils'

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
