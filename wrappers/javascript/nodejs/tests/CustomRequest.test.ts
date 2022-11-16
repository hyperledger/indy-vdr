import type { IndyVdrPool } from 'indy-vdr-shared'

import { CustomRequest } from 'indy-vdr-shared'

import { DID, setupPool } from './utils'

describe('CustomRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new CustomRequest({
      customRequest: {
        identifier: DID,
        operation: { data: 1, from: 1, type: '3', timestamp: new Date(), to: 1 },
        protocolVersion: 2,
        reqId: 2,
      },
    })

    await expect(pool.submitRequest(request)).resolves.toMatchObject({
      op: 'REPLY',
    })
  })
})
