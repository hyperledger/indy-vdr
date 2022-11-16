import type { IndyVdrPool } from 'indy-vdr-nodejs'

import { CRED_DEF_ID, setupPool } from './utils'

import { GetCredentialDefinitionRequest } from 'indy-vdr-nodejs'

describe('GetCredentialDefinitionRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetCredentialDefinitionRequest({ credentialDefinitionId: CRED_DEF_ID })

    await expect(pool.submitRequest(request)).resolves.toMatchObject({ op: 'REPLY' })
  })
})
