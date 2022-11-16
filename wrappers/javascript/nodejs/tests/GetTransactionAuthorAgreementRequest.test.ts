import type { IndyVdrPool } from 'indy-vdr-nodejs'

import { setupPool } from './utils'

import { GetTransactionAuthorAgreementRequest } from 'indy-vdr-nodejs'

describe('GetTransactionAuthorAgreementRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetTransactionAuthorAgreementRequest({})

    await expect(pool.submitRequest(request)).resolves.toMatchObject({
      op: 'REPLY',
    })
  })
})
