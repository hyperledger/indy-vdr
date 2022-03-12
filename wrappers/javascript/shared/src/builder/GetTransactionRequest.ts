import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetTransactionRequestOptions = {
  submitterDid?: string
  ledgerType: number
  seqNo: number
}

export class GetTransactionRequest extends IndyVdrRequest {
  public constructor(options: GetTransactionRequestOptions) {
    const handle = indyVdr.buildGetTxnRequest(options)
    super({ handle })
  }
}
