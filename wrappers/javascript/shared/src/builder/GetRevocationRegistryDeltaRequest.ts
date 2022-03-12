import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetRevocationRegistryDeltaRequestOptions = {
  submitterDid?: string
  revocationRegistryId: string
  fromTs?: number
  toTs: number
}

export class GetRevocationRegistryDeltaRequest extends IndyVdrRequest {
  public constructor(options: GetRevocationRegistryDeltaRequestOptions) {
    const handle = indyVdr.buildGetRevocRegDeltaRequest(options)
    super({ handle })
  }
}
