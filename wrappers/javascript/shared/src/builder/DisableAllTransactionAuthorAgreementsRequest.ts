import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type DisableAllTransactionAuthorAgreementsRequestOptions = {
  submitterDid: string
}

export class DisableAllTransactionAuthorAgreementsRequest extends IndyVdrRequest {
  public constructor(options: DisableAllTransactionAuthorAgreementsRequestOptions) {
    const handle = indyVdr.buildDisableAllTxnAuthorAgreementsRequest(options)
    super({ handle })
  }
}
