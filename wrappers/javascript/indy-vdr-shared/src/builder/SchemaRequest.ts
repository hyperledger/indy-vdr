import type { WriteRequestResponse, WriteRequestResultTxnBase } from '../types'

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

interface SchemaResultTxn extends WriteRequestResultTxnBase {
  type: '101'
  data: {
    data: {
      version: string
      attr_names: string[]
      name: string
    }
  }
}

export type SchemaResponse = WriteRequestResponse<SchemaResultTxn>

export class SchemaRequest extends IndyVdrRequest<SchemaResponse> {
  public constructor(options: SchemaRequestOptions) {
    const handle = indyVdr.buildSchemaRequest(options)
    super({ handle })
  }
}
