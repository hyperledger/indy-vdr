import type { IndyVdrPool } from 'indy-vdr-nodejs'

import { setupPool } from './utils'

import { GetAcceptanceMechanismsRequest } from 'indy-vdr-nodejs'

describe('GetAcceptanceMechanismsRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetAcceptanceMechanismsRequest({})

    await expect(pool.submitRequest(request)).resolves.toMatchObject({
      op: 'REPLY',
    })
  })
})
