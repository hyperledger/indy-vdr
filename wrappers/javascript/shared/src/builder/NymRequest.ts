import type { WriteRequestResultTxnBase, WriteRequestResponse } from '../types'

import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type NymRequestOptions = {
  submitterDid: string
  dest: string
  verkey?: string
  alias?: string
  role?: 'STEWARD' | 'TRUSTEE' | 'ENDORSER' | 'NETWORK_MONITOR'
}

interface NymResultTxn extends WriteRequestResultTxnBase {
  type: '1'
  data: {
    dest: string
    verkey: string
    alias?: string
    role?: string
  }
}

export type NymResponse = WriteRequestResponse<NymResultTxn>

export class NymRequest extends IndyVdrRequest<NymResponse> {
  public constructor(options: NymRequestOptions) {
    const handle = indyVdr.buildNymRequest(options)
    super({ handle })
  }
}
