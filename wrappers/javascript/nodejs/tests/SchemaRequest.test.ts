import type { IndyVdrPool } from 'indy-vdr-shared'

import { SchemaRequest } from 'indy-vdr-shared'

import { DID, SCHEMA_ID, setupPool } from './utils'

describe('SchemaRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new SchemaRequest({
      submitterDid: DID,
      schema: { attrNames: ['TODO'], id: SCHEMA_ID, name: 'foo', ver: '1.0', version: '1.0', seqNo: 1 },
    })

    await expect(pool.submitRequest({ requestHandle: request.handle })).rejects.toThrowError('MissingSignature()')
  })
})
