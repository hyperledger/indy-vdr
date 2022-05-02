import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetRevocationRegistryRequestOptions = {
  submitterDid?: string
  revocationRegistryId: string
  timestamp: Date
}

export type GetRevocationRegistryResponse = {
  op: string
  result: {
    revocRegDefId: string
    seqNo?: unknown
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
    txnTime?: unknown
    timestamp: number
    identifier: string
    data?: unknown
  }
}

export class GetRevocationRegistryRequest extends IndyVdrRequest {
  public constructor(options: GetRevocationRegistryRequestOptions) {
    const handle = indyVdr.buildGetRevocRegRequest(options)
    super({ handle })
  }
}
