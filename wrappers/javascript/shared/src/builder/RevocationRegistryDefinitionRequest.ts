import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type RevocationRegistryDefinitionRequestOptions = {
  submitterDid?: string
  revocationRegistryId: string
}

export class RevocationRegistryDefinitionRequest extends IndyVdrRequest {
  public constructor(options: RevocationRegistryDefinitionRequestOptions) {
    const handle = indyVdr.buildRevocRegDefRequest(options)
    super({ handle })
  }
}
