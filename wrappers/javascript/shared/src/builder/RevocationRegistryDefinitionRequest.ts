import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type RevocationRegistryDefinitionRequestOptions = {
  submitterDid: string
  revocationRegistryDefinitionV1: RevocationRegistryDefinitionV1
}

type RevocationRegistryDefinitionV1 = {
  ver: '1.0'
  id: string
  revocDefType: 'CL_ACCUM'
  tag: string
  credDefId: string
  value: RevocationRegistryDefinitionValue
}

type RevocationRegistryDefinitionValue = {
  issuanceType: 'ISSUANCE_BY_DEFAULT' | 'ISSUANCE_ON_DEMAND'
  maxCredNum: number
  publicKeys: { accumKey: string }
  tailsHash: string
  tailsLocation: string
}

export class RevocationRegistryDefinitionRequest extends IndyVdrRequest {
  public constructor(options: RevocationRegistryDefinitionRequestOptions) {
    const handle = indyVdr.buildRevocRegDefRequest(options)
    super({ handle })
  }
}
