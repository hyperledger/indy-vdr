import { IndyVdrRequest, indyVdr } from '../indyVdr'

export type RichSchemaRequestOptions = {
  submitterDid: string
  id: string
  content: string
  name: string
  version: string
  type: 'sch' | 'map' | 'ctx' | 'enc' | 'cdf' | 'pdf'
  ver: string
}

export class RichSchemaRequest extends IndyVdrRequest {
  public constructor(options: RichSchemaRequestOptions) {
    const handle = indyVdr.buildRichSchemaRequest(options)
    super({ handle })
  }
}
