import type { GetAcceptanceMechanismsResponse, IndyVdrPool } from '@hyperledger/indy-vdr-nodejs'

import { setupPool } from './utils'

import { GetAcceptanceMechanismsRequest } from '@hyperledger/indy-vdr-nodejs'

describe('GetAcceptanceMechanismsRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetAcceptanceMechanismsRequest({})
    const response: GetAcceptanceMechanismsResponse = await pool.submitRequest(request)

    expect(response).toMatchObject({
      op: 'REPLY',
    })
  })
})
