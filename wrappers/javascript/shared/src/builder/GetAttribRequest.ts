import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetAttribRequestOptions = {
  submitterDid?: string
  targetDid: string
  hash?: string
  raw?: Record<string, unknown>
  enc?: string
}

export class GetAttribRequest extends IndyVdrRequest {
  public constructor(options: GetAttribRequestOptions) {
    const handle = indyVdr.buildGetAttribRequest(options)
    super({ handle })
  }
}
