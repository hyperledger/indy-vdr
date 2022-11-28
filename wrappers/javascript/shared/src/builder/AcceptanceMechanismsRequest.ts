import { IndyVdrRequest, indyVdr } from '../indyVdr'

export type AcceptanceMechanismsRequestOptions = {
  submitterDid: string
  aml: Record<string, unknown>
  version: string
  amlContext?: string
}

// TODO: add response type. This call is probably never used, so it's not a priority.
export class AcceptanceMechanismsRequest extends IndyVdrRequest {
  public constructor(options: AcceptanceMechanismsRequestOptions) {
    const handle = indyVdr.buildAcceptanceMechanismsRequest(options)
    super({ handle })
  }
}
