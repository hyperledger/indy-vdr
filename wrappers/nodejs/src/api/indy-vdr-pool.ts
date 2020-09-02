import { rustAPI } from '../rustlib';
import { VDRInternalError } from '../errors';
import { allocateHandleBuffer, handleBufferToNumber } from './ffi-tools';
import { createFFICallbackPromise } from '../utils/ffi-helpers';
import { Callback } from 'ffi-napi';
import * as ref from 'ref-napi';
import { LedgerRequest } from './ledger-requests/ledger-request';

/**
 * @class Class representing a Indy Pool
 */
export class IndyVdrPool {
    protected _handle: Buffer;
    protected _params: string;
    protected _name: string;

    constructor(name: string, params: string) {
        this._handle = allocateHandleBuffer();
        this._params = params;
        this._name = name;
    }

    public static create(name: string, params: string): IndyVdrPool {
        try {
            const pool = new IndyVdrPool(name, params);
            rustAPI().indy_vdr_pool_create(params, pool._handle);
            return pool;
        } catch (err) {
            throw new VDRInternalError(err);
        }
    }

    public close() {
        try {
            const rc = rustAPI().indy_vdr_pool_close(this.getHandle());
            if (rc) {
                throw Error(`Failed to close pool!`);
            }
        } catch (err) {
            throw new VDRInternalError(err);
        }
    }

    public getHandle(): number {
        return handleBufferToNumber(this._handle);
    }

    public getName(): string {
        return this._name;
    }

    public getParams(): string {
        return this._params;
    }

    public async submitRequest(request: LedgerRequest): Promise<string> {
        try {
            return await createFFICallbackPromise<string>(
                (resolve, reject, cb) => {
                    const rc = rustAPI().indy_vdr_pool_submit_request(this.getHandle(), request.getHandle(), cb, 5);
                    if (rc) {
                        reject(rc);
                    }
                },
                (resolve, reject) =>
                    Callback('void', ['uint32', 'uint32', 'pointer'], (id: number, err: number, response: Buffer) => {
                        if (err) {
                            reject(err);
                            return;
                        }
                        resolve(ref.readCString(response, 0));
                    }),
            );
        } catch (err) {
            throw new VDRInternalError(err);
        }
    }

    public async getStatus(): Promise<string> {
        try {
            return await createFFICallbackPromise<string>(
                (resolve, reject, cb) => {
                    const rc = rustAPI().indy_vdr_pool_get_status(this.getHandle(), cb, 5);
                    if (rc) {
                        reject(rc);
                    }
                },
                (resolve, reject) =>
                    Callback('void', ['uint32', 'uint32', 'pointer'], (id: number, err: number, response: Buffer) => {
                        if (err) {
                            reject(err);
                            return;
                        }
                        resolve(ref.readCString(response, 0));
                    }),
            );
        } catch (err) {
            throw new VDRInternalError(err);
        }
    }

    public async getPoolTransactions(): Promise<string> {
        try {
            return await createFFICallbackPromise<string>(
                (resolve, reject, cb) => {
                    const rc = rustAPI().indy_vdr_pool_get_transactions(this.getHandle(), cb, 5);
                    if (rc) {
                        reject(rc);
                    }
                },
                (resolve, reject) =>
                    Callback('void', ['uint32', 'uint32', 'pointer'], (id: number, err: number, response: Buffer) => {
                        if (err) {
                            reject(err);
                            return;
                        }
                        resolve(ref.readCString(response, 0));
                    }),
            );
        } catch (err) {
            throw new VDRInternalError(err);
        }
    }
}
