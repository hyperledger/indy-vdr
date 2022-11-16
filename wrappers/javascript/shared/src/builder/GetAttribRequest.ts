import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetAttribRequestOptions = {
  submitterDid?: string
  targetDid: string
  hash?: string
  raw?: Record<string, unknown>
  enc?: string
}

export type GetAttribResponse = {
  op: 'REPLY'
  result: {
    reqId: number
    seqNo: unknown
    type: string
    raw: string
    state_proof: {
      proof_nodes: string
      multi_signature: {
        participants: string[]
        value: {
          timestamp: number
          state_root_hash: string
          pool_state_root_hash: string
          txn_root_hash: string
          ledger_id: number
        }
        signature: string
      }
      root_hash: string
    }
    identifier: string
    data?: unknown
    txnTime?: unknown
    dest: string
  }
}

export class GetAttribRequest extends IndyVdrRequest<GetAttribResponse> {
  public constructor(options: GetAttribRequestOptions) {
    const handle = indyVdr.buildGetAttribRequest(options)
    super({ handle })
  }
}
