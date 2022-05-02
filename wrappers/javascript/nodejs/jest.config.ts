import type { Config } from '@jest/types'

const config: Config.InitialOptions = {
  verbose: true,
  testTimeout: 120000,
}

export default config

///  xtest('Refresh pool', async () => {
///    await expect(pool.refresh()).resolves.toBe(void 0)
///  })
///
///  xtest('Submit request to pool', async () => {
///    const getSchemaRequest = new GetSchemaRequest({ schemaId: 'MrQD91rFx5mSbuhxvQCWfm:2:Identidad:0.0.1' })
///    await expect(pool.submitRequest({ requestHandle: getSchemaRequest.handle })).resolves.toMatchObject({
///      result: {
///        state_proof: {
///          multi_signature: expect.any(Object),
///          proof_nodes: expect.any(String),
///          root_hash: expect.any(String),
///        },
///        dest: 'MrQD91rFx5mSbuhxvQCWfm',
///        seqNo: 131992,
///        reqId: expect.any(Number),
///        data: {
///          version: '0.0.1',
///          name: 'Identidad',
///          attr_names: expect.any(Array),
///        },
///        txnTime: 1649767888,
///        type: '107',
///        identity: 'LibindyDid111111111111',
///      },
///      op: 'REPLY',
///    })
///  })
