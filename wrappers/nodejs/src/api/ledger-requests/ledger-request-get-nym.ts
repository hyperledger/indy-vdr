import { rustAPI } from '../../rustlib';
import { VDRInternalError } from '../../errors';
import { LedgerRequest } from './ledger-request';
import { allocCString, NULL } from 'ref-napi';

/**
 * Builds GET_NYM transaction.
 */
export class LedgerRequestGetNym extends LedgerRequest {
    constructor() {
        super();
    }

    public static create(dest: string, submitterDid?: string, seqNo?: number, timestamp?: number): LedgerRequestGetNym {
        try {
            const request = new LedgerRequestGetNym();
            const submitterDidFfi = submitterDid ? allocCString(submitterDid) : NULL;
            const seqNoFfi = seqNo ? seqNo : -1;
            const timestampFfi = timestamp ? timestamp : -1;
            rustAPI().indy_vdr_build_get_nym_request(submitterDidFfi, allocCString(dest), seqNoFfi, timestampFfi, request._handle);
            return request;
        } catch (err) {
            throw new VDRInternalError(err);
        }
    }
}
