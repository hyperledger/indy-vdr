import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type CustomRequestOptions = {
  customRequest: {
    protocolVersion: 1 | 2
    // TODO: is this optional
    reqId?: number
    identifier: string
    operation: {
      type: string
      timestamp: Date
      from: number
      to: number
    }
  }
}

export class CustomRequest extends IndyVdrRequest {
  public constructor(options: CustomRequestOptions) {
    const handle = indyVdr.buildCustomRequest(options)
    super({ handle })
  }
}
