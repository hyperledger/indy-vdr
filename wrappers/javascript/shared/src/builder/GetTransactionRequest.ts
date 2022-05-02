import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetTransactionRequestOptions = {
  submitterDid?: string
  ledgerType: number
  seqNo: number
}

export type GetTransactionResponse = {
  op: 'REPLY'
  result: {
    reqId: number
    seqNo: number
    data: {
      auditPath: Array<string>
      txnMetadata: {
        seqNo: number
      }
      txn: {
        // TODO: unknown
        metadata: Record<string, unknown>
        data: {
          dest: string
          alias: string
          verkey: string
          role: string
        }
        type: string
      }
      rootHash: string
      ver: string
      ledgerSize: number
      // TODO: unknown
      reqSignature: Record<string, unknown>
    }
    type: string
    identifier: string
    state_proof: {
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
    }
  }
}

export class GetTransactionRequest extends IndyVdrRequest {
  public constructor(options: GetTransactionRequestOptions) {
    const handle = indyVdr.buildGetTxnRequest(options)
    super({ handle })
  }
}
