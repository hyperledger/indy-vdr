import type { GetTransactionAuthorAgreementResponse, IndyVdrPool } from 'indy-vdr-shared'

import { GetTransactionAuthorAgreementRequest } from 'indy-vdr-shared'

import { setupPool } from './utils'

describe('GetTransactionAuthorAgreementRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetTransactionAuthorAgreementRequest({})

    await expect(
      pool.submitRequest<GetTransactionAuthorAgreementResponse>({ requestHandle: request.handle })
    ).resolves.toMatchObject({ op: 'REPLY' })
  })
})
