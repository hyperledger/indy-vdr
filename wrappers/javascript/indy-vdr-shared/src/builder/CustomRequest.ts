import { indyVdr, IndyVdrRequest } from '../indyVdr'

// TODO: this needs some more work, but need to find a way to use it first.
export type CustomRequestOptions = {
  customRequest: {
    protocolVersion: 1 | 2
    reqId?: number
    identifier: string
    operation: Record<string, unknown>
  }
}

export class CustomRequest extends IndyVdrRequest {
  public constructor(options: CustomRequestOptions) {
    const handle = indyVdr.buildCustomRequest(options)
    super({ handle })
  }
}
