import { indyVdr, IndyVdrRequest } from '../indyVdr'
import { serializeArguments, SerializedOptions } from '../utils/serialize'

export type CredentialDefinitionRequestOptions = {
  submitterDid: string
  credentialDefinition: string
}

export class CredentialDefinitionRequest extends IndyVdrRequest {
  public constructor(options: CredentialDefinitionRequestOptions) {
    const { credentialDefinition, submitterDid } = serializeArguments(options) as SerializedOptions<typeof options>
    const handle = indyVdr.buildCredDefRequest({ submitterDid, credentialDefinition: credentialDefinition })
    super({ handle })
  }
}
