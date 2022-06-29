import { GetRichSchemaObjectByIdRequest, IndyVdrError } from 'indy-vdr-shared'

import { DID, SCHEMA_ID, setupPool } from './utils'

describe('GetRichSchemaObjectByIdRequest', () => {
  beforeAll(() => setupPool())

  test('Submit request', () => {
    try {
      new GetRichSchemaObjectByIdRequest({ submitterDid: DID, id: SCHEMA_ID })
    } catch (e) {
      expect(e).toBeInstanceOf(IndyVdrError)
    }
  })
})
