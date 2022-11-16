import type { IndyVdrPool } from 'indy-vdr-shared'

import { DisableAllTransactionAuthorAgreementsRequest } from 'indy-vdr-shared'

import { DID, setupPool } from './utils'

describe('DisableAllTransactionsAuthorAgreementRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new DisableAllTransactionAuthorAgreementsRequest({ submitterDid: DID })

    await expect(pool.submitRequest(request)).rejects.toThrowError('MissingSignature()')
  })
})
