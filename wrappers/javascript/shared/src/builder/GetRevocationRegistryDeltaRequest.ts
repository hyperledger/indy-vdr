import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetRevocationRegistryDeltaRequestOptions = {
  submitterDid?: string
  revocationRegistryId: string
  fromTs?: number
  toTs: number
}

export type GetRevocationRegistryDeltaResponse = {
  op: string
  result: {
    revocRegDefId: string
    to: number
    type: string
    reqId: number
    txnTime?: unknown
    seqNo?: unknown
    identifier: string
    data?: unknown
  }
}

export class GetRevocationRegistryDeltaRequest extends IndyVdrRequest {
  public constructor(options: GetRevocationRegistryDeltaRequestOptions) {
    const handle = indyVdr.buildGetRevocRegDeltaRequest(options)
    super({ handle })
  }
}
