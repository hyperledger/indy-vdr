import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetSchemaRequestOptions = {
  submitterDid?: string
  schemaId: string
}

export class GetSchemaRequest extends IndyVdrRequest {
  public constructor(options: GetSchemaRequestOptions) {
    const handle = indyVdr.buildGetSchemaRequest(options)
    super({ handle })
  }
}
