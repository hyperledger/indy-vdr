import type { GetTransactionResponse, IndyVdrPool } from 'indy-vdr-shared'

import { GetTransactionRequest } from 'indy-vdr-shared'

import { setupPool } from './utils'

describe('GetTransactionRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetTransactionRequest({ ledgerType: 1, seqNo: 1 })

    await expect(pool.submitRequest<GetTransactionResponse>({ requestHandle: request.handle })).resolves.toMatchObject({
      op: 'REPLY',
    })
  })
})
