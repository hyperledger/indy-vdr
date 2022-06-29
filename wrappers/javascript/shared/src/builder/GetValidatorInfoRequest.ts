import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetValidatorInfoRequestOptions = {
  submitterDid: string
}

export class GetValidatorInfoRequest extends IndyVdrRequest {
  public constructor(options: GetValidatorInfoRequestOptions) {
    const handle = indyVdr.buildGetValidatorInfoRequest(options)
    super({ handle })
  }
}
