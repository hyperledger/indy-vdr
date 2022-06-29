import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type CredentialDefinitionRequestOptions = {
  submitterDid: string
  credentialDefinition: {
    ver: '1.0'
    id: string
    schemaId: string
    type: 'CL'
    tag: string
    value: {
      primary: Record<string, unknown>
      revocation?: unknown
    }
  }
}

export class CredentialDefinitionRequest extends IndyVdrRequest {
  public constructor(options: CredentialDefinitionRequestOptions) {
    const handle = indyVdr.buildCredDefRequest(options)
    super({ handle })
  }
}
