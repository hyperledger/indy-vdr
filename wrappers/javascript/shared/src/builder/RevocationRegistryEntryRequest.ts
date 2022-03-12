import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type RevocationRegistryEntryRequestOptions = {
  submitterDid: string
  revocationRegistryDefinitionId: string
  revocationRegistryDefinitionType: string
  revocationRegistryEntry: {
    ver: '1.0'
    value: string
  }
}

export class RevocationRegistryEntryRequest extends IndyVdrRequest {
  public constructor(options: RevocationRegistryEntryRequestOptions) {
    const handle = indyVdr.buildRevocRegEntryRequest(options)
    super({ handle })
  }
}
