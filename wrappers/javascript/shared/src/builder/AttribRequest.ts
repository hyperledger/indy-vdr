import { IndyVdrRequest, indyVdr } from '../indyVdr'

export type AttribRequestOptions = {
  submitterDid: string
  targetDid: string
  hash?: string
  raw?: string
  enc?: string
}

export class AttribRequest extends IndyVdrRequest {
  public constructor(options: AttribRequestOptions) {
    const handle = indyVdr.buildAttribRequest(options)
    super({ handle })
  }
}
