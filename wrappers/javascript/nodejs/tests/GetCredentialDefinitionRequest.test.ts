import type { GetCredentialDefinitionReponse, IndyVdrPool } from 'indy-vdr-shared'

import { GetCredentialDefinitionRequest } from 'indy-vdr-shared'

import { CRED_DEF_ID, setupPool } from './utils'

describe('GetCredentialDefinitionRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetCredentialDefinitionRequest({ credentialDefinitionId: CRED_DEF_ID })

    await expect(
      pool.submitRequest<GetCredentialDefinitionReponse>({ requestHandle: request.handle })
    ).resolves.toMatchObject({ op: 'REPLY' })
  })
})
