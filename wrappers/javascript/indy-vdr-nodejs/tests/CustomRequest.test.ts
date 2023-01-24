import type { IndyVdrPool } from '@hyperledger/indy-vdr-nodejs'

import { DID, setupPool } from './utils'

import { CustomRequest } from '@hyperledger/indy-vdr-nodejs'

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
