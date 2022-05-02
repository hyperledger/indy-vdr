import type { GetAttribResponse, IndyVdrPool } from 'indy-vdr-shared'

import { GetAttribRequest } from 'indy-vdr-shared'

import { DID, setupPool } from './utils'

describe('GetAttribRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetAttribRequest({ targetDid: DID, raw: { TODO: { TODO: 'TODO' } } })

    await expect(pool.submitRequest<GetAttribResponse>({ requestHandle: request.handle })).resolves.toMatchObject({
      op: 'REPLY',
    })
  })
})
