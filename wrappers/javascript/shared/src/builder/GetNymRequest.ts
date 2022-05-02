import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetNymRequestOptions = {
  submitterDid?: string
  dest: string
}

export type GetNymResponse = {
  op: 'REPLY'
  result: {
    reqId: number
    seqNo: number
    data: string
    txnTime: number
    identifier: string
    type: string
    dest: string
    state_proof: {
      root_hash: string
      multi_signature: {
        value: {
          pool_state_root_hash: string
          state_root_hash: string
          ledger_id: number
          timestamp: number
          txn_root_hash: string
        }
        participants: Array<string>
        signature: string
      }
      proof_nodes: string
    }
  }
}

export class GetNymRequest extends IndyVdrRequest {
  public constructor(options: GetNymRequestOptions) {
    const handle = indyVdr.buildGetNymRequest(options)
    super({ handle })
  }
}
