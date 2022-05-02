/* eslint-disable @typescript-eslint/no-unsafe-assignment */
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

    await expect(pool.submitRequest({ requestHandle: request.handle })).resolves.toMatchObject({
      op: 'REPLY',
      result: {
        data: {
          auditPath: expect.any(Array),
          ledgerSize: 33356,
          reqSignature: {},
          rootHash: 'Hvoo4x4Jv7DF95WVsUEDJKJxTDbNZNCNMbJtn9A91YAc',
          txn: expect.any(Object),
          txnMetadata: {
            seqNo: 1,
          },
          ver: '1',
        },
        identifier: 'PqdUtwWhuX4GWRa58WSdvn',
        reqId: 2,
        seqNo: 1,
        state_proof: expect.any(Object),
        type: '3',
      },
    })
  })
})
