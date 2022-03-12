import { IndyVdrRequest, indyVdr } from '../indyVdr'

export type TransactionAuthorAgreementRequestOptions = {
  submitterDid: string
  text?: string
  version: string
  ratificationTs?: number
  retirementTs?: number
}

export class TransactionAuthorAgreementRequest extends IndyVdrRequest {
  public constructor(options: TransactionAuthorAgreementRequestOptions) {
    const handle = indyVdr.buildTxnAuthorAgreementRequest(options)
    super({ handle })
  }
}
