import { IndyVdrRequest, indyVdr } from '../indyVdr'

export type SchemaRequestOptions = {
  submitterDid: string
  schema: {
    id: string
    name: string
    version: string
    attrNames: string[]
    seqNo?: number
    ver: '1.0'
  }
}

export class SchemaRequest extends IndyVdrRequest {
  public constructor(options: SchemaRequestOptions) {
    const handle = indyVdr.buildSchemaRequest(options)
    super({ handle })
  }
}
