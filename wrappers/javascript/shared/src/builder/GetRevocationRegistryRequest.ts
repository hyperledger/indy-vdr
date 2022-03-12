import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetRevocationRegistryRequestOptions = {
  submitterDid?: string
  revocationRegistryId: string
  timestamp: Date
}

export class GetRevocationRegistryRequest extends IndyVdrRequest {
  public constructor(options: GetRevocationRegistryRequestOptions) {
    const handle = indyVdr.buildGetRevocRegRequest(options)
    super({ handle })
  }
}
