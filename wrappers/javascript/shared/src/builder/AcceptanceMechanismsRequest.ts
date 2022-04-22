import { IndyVdrRequest, indyVdr } from '../indyVdr'

export type AcceptanceMechanismsRequestOptions = {
  submitterDid: string
  aml: Record<string, unknown>
  version: string
  amlContext?: string
}

export class AcceptanceMechanismsRequest extends IndyVdrRequest {
  public constructor(options: AcceptanceMechanismsRequestOptions) {
    const handle = indyVdr.buildAcceptanceMechanismsRequest(options)
    super({ handle })
  }
}
