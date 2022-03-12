import type { PoolStatus, SubmitAction, SubmitRequest, Transactions, Verifiers } from '../types'

import { indyVdr } from './indyVdr'

export type PoolSubmitRequestOptions = {
  requestHandle: number
}

export type PoolSubmitActionOptions = {
  requestHandle: number
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
    return indyVdr.poolGetStatus(this._handle)
  }

  public get verifiers(): Promise<Verifiers> {
    return indyVdr.poolGetVerifiers(this._handle)
  }

  public get transactions(): Promise<Transactions> {
    return indyVdr.poolGetTransactions(this._handle)
  }

  public async refresh(): Promise<void> {
    await indyVdr.poolRefresh(this._handle)
  }

  public close(): void {
    indyVdr.poolClose(this._handle)
  }

  public async submitAction(options: PoolSubmitActionOptions): Promise<SubmitAction> {
    return indyVdr.poolSubmitAction(this._handle, options)
  }

  public async submitRequest(options: PoolSubmitRequestOptions): Promise<SubmitRequest> {
    return indyVdr.poolSubmitRequest(this._handle, options)
  }
}
