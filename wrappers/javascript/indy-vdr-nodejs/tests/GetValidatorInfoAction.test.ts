import type { GetValidatorInfoResponse, IndyVdrPool } from '@hyperledger/indy-vdr-nodejs'

import { DID, setupPool } from './utils'

import { GetValidatorInfoAction } from '@hyperledger/indy-vdr-nodejs'

describe('GetValidatorInfoAction', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit action', async () => {
    const action = new GetValidatorInfoAction({ submitterDid: DID })
    const response: GetValidatorInfoResponse = await pool.submitAction(action)

    expect(response).toMatchObject({})
  })
})
