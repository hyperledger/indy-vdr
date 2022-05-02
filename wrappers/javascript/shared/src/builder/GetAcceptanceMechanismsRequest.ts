import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetAcceptanceMechanismsRequestOptions = {
  timestamp?: Date
  submitterDid?: string
  version?: string
}

export type GetAcceptanceMechanismsResponse = {
  op: 'REPLY'
  result: {
    reqId: number
    seqNo: number
    type: string
    state_proof: {
      proof_nodes: string
      multi_signature: string[]
      root_hash: string
    }
    data: {
      version: string
      amlContext: string
      aml: string[]
    }
    txnTime: number
    identifier: string
  }
}

export class GetAcceptanceMechanismsRequest extends IndyVdrRequest {
  public constructor(options: GetAcceptanceMechanismsRequestOptions) {
    const handle = indyVdr.buildGetAcceptanceMechanismsRequest(options)
    super({ handle })
  }
}
