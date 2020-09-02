import { rustAPI } from '../../rustlib';
import { VDRInternalError } from '../../errors';
import { LedgerRequest } from './ledger-request';

export class LedgerRequestCustom extends LedgerRequest {
    protected _params: string;

    constructor(params: string) {
        super();
        this._params = params;
    }

    public static create(params: string): LedgerRequestCustom {
        try {
            const request = new LedgerRequestCustom(params);
            rustAPI().indy_vdr_build_custom_request(params, request._handle);
            return request;
        } catch (err) {
            throw new VDRInternalError(err);
        }
    }
}
