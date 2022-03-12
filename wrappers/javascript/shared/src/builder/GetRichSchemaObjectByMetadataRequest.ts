import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetRichSchemaObjectByMetadataRequestOptions = {
  submitterDid: string
  type: string
  name: string
  version: string
}

export class GetRichSchemaObjectByMetadataRequest extends IndyVdrRequest {
  public constructor(options: GetRichSchemaObjectByMetadataRequestOptions) {
    const handle = indyVdr.buildGetRichSchemaObjectByMetadataRequest(options)
    super({ handle })
  }
}
