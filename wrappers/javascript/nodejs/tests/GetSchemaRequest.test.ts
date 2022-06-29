import type { GetSchemaResponse, IndyVdrPool } from 'indy-vdr-shared'

import { GetSchemaRequest } from 'indy-vdr-shared'

import { SCHEMA_ID, setupPool } from './utils'

describe('GetSchemaRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetSchemaRequest({
      schemaId: SCHEMA_ID,
    })

    await expect(pool.submitRequest<GetSchemaResponse>({ requestHandle: request.handle })).resolves.toMatchObject({
      op: 'REPLY',
    })
  })
})
