import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type CustomRequestOptions = {
  customRequest: {
    protocolVersion: 1 | 2
    reqId?: number
    identifier: string
    operation: {
      // TODO: unsure about string
      data: number
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
