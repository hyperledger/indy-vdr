import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type NymRequestOptions = {
  submitterDid: string
  dest: string
  verkey?: string
  alias?: string
  role?: 'STEWARD' | 'TRUSTEE' | 'ENDORSER' | 'NETWORK_MONITOR'
}

export class NymRequest extends IndyVdrRequest {
  public constructor(options: NymRequestOptions) {
    const handle = indyVdr.buildNymRequest(options)
    super({ handle })
  }
}
