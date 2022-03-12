import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetCredentialDefinitionRequestOptions = {
  submitterDid?: string
  credentialDefinitionId: string
}

export class GetCredentialDefinitionRequest extends IndyVdrRequest {
  public constructor(options: GetCredentialDefinitionRequestOptions) {
    const handle = indyVdr.buildGetCredDefRequest(options)
    super({ handle })
  }
}
