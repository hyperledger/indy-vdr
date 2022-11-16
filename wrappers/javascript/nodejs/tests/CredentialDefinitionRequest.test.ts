import type { IndyVdrPool } from 'indy-vdr-shared'

import { CredentialDefinitionRequest } from 'indy-vdr-shared'

import { DID, setupPool } from './utils'

describe('CredentialDefinitionRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new CredentialDefinitionRequest({
      credentialDefinition: {
        ver: '1.0',
        id: 'TODO',
        schemaId: '1',
        type: 'CL',
        tag: 'TODO',
        value: {
          primary: {
            TODO: 'TODO',
          },
        },
      },
      submitterDid: DID,
    })

    await expect(pool.submitRequest(request)).rejects.toThrowError('MissingSignature()')
  })
})
