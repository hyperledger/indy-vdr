import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetCredentialDefinitionRequestOptions = {
  submitterDid?: string
  credentialDefinitionId: string
}

export type GetCredentialDefinitionReponse = {
  op: 'REPLY'
  result: {
    origin: string
    signature_type: string
    seqNo?: unknown
    identifier: string
    data?: unknown
    txnTime?: unknown
    ref: number
    tag: string
    type: string
    reqId: number
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
  }
}

export class GetCredentialDefinitionRequest extends IndyVdrRequest {
  public constructor(options: GetCredentialDefinitionRequestOptions) {
    const handle = indyVdr.buildGetCredDefRequest(options)
    super({ handle })
  }
}
