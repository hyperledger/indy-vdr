import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type CredentialDefinitionRequestOptions = {
  submitterDid: string
  credentialDefinition: string
}

export class CredentialDefinitionRequest extends IndyVdrRequest {
  public constructor(options: CredentialDefinitionRequestOptions) {
    const handle = indyVdr.buildCredDefRequest(options)
    super({ handle })
  }
}
