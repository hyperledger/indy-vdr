import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetRevocationRegistryDefinitionRequestOptions = {
  submitterDid?: string
  revocationRegistryId: string
}

export type GetRevocationRegistryDefinitionResponse = {
  op: 'REPLY'
  result: {
    txnTime?: unknown
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
    id: string
    seqNo?: unknown
    identifier: string
    data?: unknown
  }
}

export class GetRevocationRegistryDefinitionRequest extends IndyVdrRequest<GetRevocationRegistryDefinitionResponse> {
  public constructor(options: GetRevocationRegistryDefinitionRequestOptions) {
    const handle = indyVdr.buildGetRevocRegDefRequest(options)
    super({ handle })
  }
}
