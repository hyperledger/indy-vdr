/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import type { GetAcceptanceMechanismsResponse, IndyVdrPool } from 'indy-vdr-shared'

import { GetAcceptanceMechanismsRequest } from 'indy-vdr-shared'

import { setupPool } from './utils'

describe('GetAcceptanceMechanismsRequest', () => {
  let pool: IndyVdrPool

  beforeAll(() => (pool = setupPool()))

  test('Submit request', async () => {
    const request = new GetAcceptanceMechanismsRequest({})

    await expect(
      pool.submitRequest<GetAcceptanceMechanismsResponse>({ requestHandle: request.handle })
    ).resolves.toMatchObject({
      op: 'REPLY',
    })
  })
})
