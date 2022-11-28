import type { GetAttribResponse, IndyVdrPool } from 'indy-vdr-nodejs'

import { DID, setupPool } from './utils'

import { GetAttribRequest } from 'indy-vdr-nodejs'

describe('GetAttribRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetAttribRequest({ targetDid: DID, raw: 'endpoint' })
    const response: GetAttribResponse = await pool.submitRequest(request)

    expect(response).toMatchObject({
      op: 'REPLY',
    })
  })
})
