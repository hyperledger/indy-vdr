import type { GetTransactionAuthorAgreementResponse, IndyVdrPool } from '@hyperledger/indy-vdr-nodejs'

import { setupPool } from './utils'

import { GetTransactionAuthorAgreementRequest } from '@hyperledger/indy-vdr-nodejs'

describe('GetTransactionAuthorAgreementRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetTransactionAuthorAgreementRequest({})
    const response: GetTransactionAuthorAgreementResponse = await pool.submitRequest(request)

    expect(response).toMatchObject({
      op: 'REPLY',
    })
  })
})
