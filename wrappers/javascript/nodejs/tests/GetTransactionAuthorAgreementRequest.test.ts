import type { IndyVdrPool } from 'indy-vdr-shared'

import { GetTransactionAuthorAgreementRequest } from 'indy-vdr-shared'

import { setupPool } from './utils'

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
