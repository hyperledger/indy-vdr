import type { GetSchemaResponse, IndyVdrPool } from '@hyperledger/indy-vdr-nodejs'

import { SCHEMA_ID, setupPool } from './utils'

import { GetSchemaRequest } from '@hyperledger/indy-vdr-nodejs'

describe('GetSchemaRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetSchemaRequest({
      schemaId: SCHEMA_ID,
    })
    const response: GetSchemaResponse = await pool.submitRequest(request)

    expect(response).toMatchObject({
      op: 'REPLY',
    })
  })
})
