import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetRichSchemaObjectByIdRequestOptions = {
  submitterDid: string
  id: string
}

export class GetRichSchemaObjectByIdRequest extends IndyVdrRequest {
  public constructor(options: GetRichSchemaObjectByIdRequestOptions) {
    const handle = indyVdr.buildGetRichSchemaObjectByIdRequest(options)
    super({ handle })
  }
}
