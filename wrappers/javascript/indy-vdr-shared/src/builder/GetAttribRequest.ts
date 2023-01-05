import type { GetRequestResultFoundBase, GetRequestResultNotFoundBase, GetRequestResponse } from '../types'

import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type GetAttribRequestOptions = {
  submitterDid?: string
  targetDid: string
  hash?: string
  raw?: string
  enc?: string
}

interface GetAttribFoundResult extends GetRequestResultFoundBase {
  type: '104'
  dest: string
  data: string
  raw?: string
}

interface GetAttribNotFoundResult extends GetRequestResultNotFoundBase {
  type: '104'
  data: null
  dest: string
  raw?: string
}

export type GetAttribResponse = GetRequestResponse<GetAttribFoundResult, GetAttribNotFoundResult>

export class GetAttribRequest extends IndyVdrRequest<GetAttribResponse> {
  public constructor(options: GetAttribRequestOptions) {
    const handle = indyVdr.buildGetAttribRequest(options)
    super({ handle })
  }
}
