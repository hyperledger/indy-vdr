import type { PoolStatus, SubmitAction, SubmitRequest, Transactions, Verifiers } from '../types'

import { indyVdr } from './indyVdr'
import { RequestHandle } from './IndyVdrNativeBindings'

export type PoolSubmitRequestOptions = {
  // poolHandle: PoolHandle
  requestHandle: RequestHandle
}

export type PoolSubmitActionOptions = {
  // poolHandle: PoolHandle
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
    indyVdr.poolClose({ poolHandle: this.handle })
  }

  public async submitAction(options: PoolSubmitActionOptions): Promise<SubmitAction> {
    return indyVdr.poolSubmitAction({ poolHandle: this.handle, ...options })
  }

  public async submitRequest(options: PoolSubmitRequestOptions): Promise<SubmitRequest> {
    return indyVdr.poolSubmitRequest({ poolHandle: this.handle, ...options })
  }
}
