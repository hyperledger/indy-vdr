import type { GetTransactionResponse, IndyVdrPool } from '@hyperledger/indy-vdr-nodejs'

import { setupPool } from './utils'

import { GetTransactionRequest } from '@hyperledger/indy-vdr-nodejs'

describe('GetTransactionRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetTransactionRequest({ ledgerType: 1, seqNo: 1 })
    const response: GetTransactionResponse = await pool.submitRequest(request)

    expect(response).toMatchObject({
      op: 'REPLY',
    })
  })
})
