import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetNymRequestOptions = {
  submitterDid?: string
  dest: string
}

export class GetNymRequest extends IndyVdrRequest {
  public constructor(options: GetNymRequestOptions) {
    const handle = indyVdr.buildGetNymRequest(options)
    super({ handle })
  }
}
