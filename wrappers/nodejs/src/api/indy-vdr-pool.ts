import { rustAPI } from '../rustlib';
import { VDRInternalError } from '../errors';
import { allocateHandleBuffer, handleBufferToNumber } from './ffi-tools';

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

    public getHandle(): number {
        return handleBufferToNumber(this._handle);
    }

    public getName(): string {
        return this._name;
    }

    public getParams(): string {
        return this._params;
    }
}
