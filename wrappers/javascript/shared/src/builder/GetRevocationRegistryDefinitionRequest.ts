import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetRevocationRegistryDefinitionRequestOptions = {
  submitterDid?: string
  revocationRegistryId: string
}

export class GetRevocationRegistryDefinitionRequest extends IndyVdrRequest {
  public constructor(options: GetRevocationRegistryDefinitionRequestOptions) {
    const handle = indyVdr.buildGetRevocRegDefRequest(options)
    super({ handle })
  }
}
