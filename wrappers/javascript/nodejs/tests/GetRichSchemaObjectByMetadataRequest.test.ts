import { IndyVdrError, GetRichSchemaObjectByMetadataRequest } from 'indy-vdr-shared'

import { DID, setupPool } from './utils'

describe('GetRichSchemaObjectByMetadataRequest', () => {
  beforeAll(() => setupPool())

  test('Submit request', () => {
    try {
      new GetRichSchemaObjectByMetadataRequest({
        submitterDid: DID,
        type: 'TODO',
        name: 'TODO',
        version: 'TODO',
      })
    } catch (e) {
      expect(e).toBeInstanceOf(IndyVdrError)
    }
  })
})
