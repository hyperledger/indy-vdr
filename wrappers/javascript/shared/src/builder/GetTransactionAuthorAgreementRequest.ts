import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetTransactionAuthorAgreementRequestOptions = {
  submitterDid?: string
  data?: string
}

export type GetTransactionAuthorAgreementResponse = {
  op: 'REPLY'
  result: {
    reqId: number
    seqNo: number
    type: string
    state_proof: {
      proof_nodes: string
      multi_signature: {
        participants: Array<string>
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
    data: {
      ratification_ts: number
      digest: string
      version: string
      text: string
    }
    txnTime: number
    identifier: string
  }
}

export class GetTransactionAuthorAgreementRequest extends IndyVdrRequest {
  public constructor(options: GetTransactionAuthorAgreementRequestOptions) {
    const handle = indyVdr.buildGetTxnAuthorAgreementRequest(options)
    super({ handle })
  }
}
