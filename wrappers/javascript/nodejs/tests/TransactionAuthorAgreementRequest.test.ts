import type { IndyVdrPool } from 'indy-vdr-shared'

import { TransactionAuthorAgreementRequest } from 'indy-vdr-shared'

import { DID, setupPool } from './utils'

describe('TransactionAuthorAgreementRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new TransactionAuthorAgreementRequest({
      submitterDid: DID,
      version: 'TODO',
    })

    await expect(pool.submitRequest({ requestHandle: request.handle })).rejects.toThrowError('MissingSignature()')
  })
})
