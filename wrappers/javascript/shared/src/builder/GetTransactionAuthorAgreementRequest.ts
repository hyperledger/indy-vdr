import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetTransactionAuthorAgreementRequestOptions = {
  submitterDid?: string
  data?: string
}

export class GetTransactionAuthorAgreementRequest extends IndyVdrRequest {
  public constructor(options: GetTransactionAuthorAgreementRequestOptions) {
    const handle = indyVdr.buildGetTxnAuthorAgreementRequest(options)
    super({ handle })
  }
}
