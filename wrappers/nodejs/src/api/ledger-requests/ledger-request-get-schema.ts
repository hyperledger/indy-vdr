import { rustAPI } from '../../rustlib';
import { VDRInternalError } from '../../errors';
import { LedgerRequest } from './ledger-request';
import { allocCString, NULL } from 'ref-napi';

export class LedgerRequestGetSchema extends LedgerRequest {
    constructor() {
        super();
    }

    public static create(schemaId: string, submitterDid?: string): LedgerRequestGetSchema {
        try {
            const request = new LedgerRequestGetSchema();
            const submitterDidFfi = submitterDid ? allocCString(submitterDid) : NULL;
            rustAPI().indy_vdr_build_get_schema_request(submitterDidFfi, allocCString(schemaId), request._handle);
            return request;
        } catch (err) {
            throw new VDRInternalError(err);
        }
    }
}
