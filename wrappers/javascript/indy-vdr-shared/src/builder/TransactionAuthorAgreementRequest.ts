import { IndyVdrRequest, indyVdr } from '../indyVdr'

export type TransactionAuthorAgreementRequestOptions = {
  submitterDid: string
  text?: string
  version: string
  ratificationTs?: number
  retirementTs?: number
}

// TODO: add response type. This call is probably never used, so it's not a priority.
export class TransactionAuthorAgreementRequest extends IndyVdrRequest {
  public constructor(options: TransactionAuthorAgreementRequestOptions) {
    const handle = indyVdr.buildTxnAuthorAgreementRequest(options)
    super({ handle })
  }
}
