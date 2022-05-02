import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetSchemaRequestOptions = {
  submitterDid?: string
  schemaId: string
}

export type GetSchemaResponse = {
  op: 'REPLY'
  result: {
    txnTime: number
    seqNo: number
    state_proof: {
      multi_signature: {
        signature: string
        value: {
          ledger_id: number
          state_root_hash: string
          timestamp: number
          txn_root_hash: string
          pool_state_root_hash: string
        }
        participants: Array<string>
      }
      root_hash: string
      proof_nodes: string
    }
    type: string
    reqId: number
    dest: string
    identifier: string
    data: {
      attr_names: Array<string>
      name: string
      version: string
    }
  }
}

export class GetSchemaRequest extends IndyVdrRequest {
  public constructor(options: GetSchemaRequestOptions) {
    const handle = indyVdr.buildGetSchemaRequest(options)
    super({ handle })
  }
}
