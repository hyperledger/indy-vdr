import { indyVdr, IndyVdrRequest } from '../indyVdr'

export type CustomRequestOptions = {
  customRequest: string | Record<string, unknown>
}

export class CustomRequest extends IndyVdrRequest {
  public constructor(options: CustomRequestOptions) {
    const handle = indyVdr.buildCustomRequest(options)
    super({ handle })
  }
}
