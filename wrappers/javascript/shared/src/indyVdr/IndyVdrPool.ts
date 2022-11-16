import type { PoolStatus, Transactions, Verifiers } from '../types'
import type { RequestHandle } from './IndyVdrNativeBindings'
import type { IndyVdrRequest, RequestResponseType } from './IndyVdrRequest'

import { indyVdr } from './indyVdr'

export type PoolSubmitRequestOptions = {
  requestHandle: RequestHandle
}

export type PoolSubmitActionOptions = {
  requestHandle: RequestHandle
  nodes?: string[]
  timeout?: number
}

export class IndyVdrPool {
  private _handle: number

  public constructor(options: { handle: number }) {
    const { handle } = options

    this._handle = handle
  }

  public get handle(): number {
    return this._handle
  }

  public get status(): Promise<PoolStatus> {
    return indyVdr.poolGetStatus({ poolHandle: this.handle })
  }

  public get verifiers(): Promise<Verifiers> {
    return indyVdr.poolGetVerifiers({ poolHandle: this.handle })
  }

  public get transactions(): Promise<Transactions> {
    return indyVdr.poolGetTransactions({ poolHandle: this.handle })
  }

  public async refresh(): Promise<void> {
    await indyVdr.poolRefresh({ poolHandle: this.handle })
  }

  public close(): void {
    return indyVdr.poolClose({ poolHandle: this.handle })
  }

  public async submitAction<T extends Record<string, unknown>>(options: PoolSubmitActionOptions): Promise<T> {
    return indyVdr.poolSubmitAction<T>({ poolHandle: this.handle, ...options })
  }

  public async submitRequest<Request extends IndyVdrRequest>(request: Request): Promise<RequestResponseType<Request>> {
    return indyVdr.poolSubmitRequest({
      poolHandle: this.handle,
      requestHandle: request.handle,
    })
  }
}
