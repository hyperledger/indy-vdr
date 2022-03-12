import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetAcceptanceMechanismsRequestOptions = {
  timestamp?: Date
  submitterDid?: string
  version?: string
}

export class GetAcceptanceMechanismsRequest extends IndyVdrRequest {
  public constructor(options: GetAcceptanceMechanismsRequestOptions) {
    const handle = indyVdr.buildGetAcceptanceMechanismsRequest(options)
    super({ handle })
  }
}
