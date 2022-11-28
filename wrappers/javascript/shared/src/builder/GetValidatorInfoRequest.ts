import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetValidatorInfoRequestOptions = {
  submitterDid: string
}

export type GetValidatorInfoResponse = Record<string, string>

export class GetValidatorInfoRequest extends IndyVdrRequest<GetValidatorInfoResponse> {
  public constructor(options: GetValidatorInfoRequestOptions) {
    const handle = indyVdr.buildGetValidatorInfoRequest(options)
    super({ handle })
  }
}
