import { IndyVdrRequest, indyVdr } from '../indyVdr'

export type AcceptanceMechanismsRequestOptions = {
  submitterDid: string
  // TODO: or Record<string, unknown>
  aml: string
  version: string
  amlContext?: string
}

export class AcceptanceMechanismsRequest extends IndyVdrRequest {
  public constructor(options: AcceptanceMechanismsRequestOptions) {
    const handle = indyVdr.buildAcceptanceMechanismsRequest(options)
    super({ handle })
  }
}
