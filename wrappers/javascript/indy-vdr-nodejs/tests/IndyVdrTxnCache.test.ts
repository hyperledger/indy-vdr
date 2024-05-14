import type { GetCredentialDefinitionResponse, GetSchemaResponse, IndyVdrPool } from '@hyperledger/indy-vdr-nodejs'

import { CRED_DEF_ID, genesisTxnPath, SCHEMA_ID } from './utils'

import { PoolCreate } from '@hyperledger/indy-vdr-nodejs'
import { GetCredentialDefinitionRequest, GetSchemaRequest, indyVdr } from '@hyperledger/indy-vdr-shared'
import { accessSync, rmSync } from 'fs';

describe('IndyVdrTxnCache', () => {
  let pool: IndyVdrPool

  beforeAll(() => {
    pool = new PoolCreate({ parameters: { transactions_path: genesisTxnPath } })
    indyVdr.setLedgerTxnCache({ capacity: 100, expiry_offset_ms: 60 * 60 * 1000, path: "txn-cache" })
  })

  afterAll(() => {
    rmSync('txn-cache', { force: true, recursive: true })
  })

  test('Get pool handle', () => {
    let accessed = false
    try {
      accessSync('txn-cache')
      accessed = true
    } catch { }
    expect(accessed).toBe(true)
  })

  test('Submit Schema request', async () => {
    const request = new GetSchemaRequest({
      schemaId: SCHEMA_ID,
    })
    const response: GetSchemaResponse = await pool.submitRequest(request)

    const requestCached = new GetSchemaRequest({
      schemaId: SCHEMA_ID,
    })
    const responseCached: GetSchemaResponse = await pool.submitRequest(requestCached)

    expect(response).toMatchObject({
      op: 'REPLY',
    })
    expect(response).toMatchObject({ ...responseCached, result: { ...responseCached.result, reqId: expect.any(Number) } })
  })

  test('Submit Cred Def request', async () => {
    const request = new GetCredentialDefinitionRequest({ credentialDefinitionId: CRED_DEF_ID })
    const response: GetCredentialDefinitionResponse = await pool.submitRequest(request)

    const requestCached = new GetCredentialDefinitionRequest({ credentialDefinitionId: CRED_DEF_ID })
    const responseCached: GetCredentialDefinitionResponse = await pool.submitRequest(requestCached)

    expect(response).toMatchObject({
      op: 'REPLY',
    })
    expect(response).toMatchObject({ ...responseCached, result: { ...responseCached.result, reqId: expect.any(Number) } })
  })

})
