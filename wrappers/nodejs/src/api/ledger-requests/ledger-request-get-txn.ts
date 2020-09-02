import { rustAPI } from '../../rustlib';
import { VDRInternalError } from '../../errors';
import { LedgerRequest } from './ledger-request';
import { allocCString, NULL } from 'ref-napi';

export class LedgerRequestGetTxn extends LedgerRequest {
    constructor() {
        super();
    }

    public static create(ledgerType: number, seqNo: number, submitterDid?: string): LedgerRequestGetTxn {
        try {
            const request = new LedgerRequestGetTxn();
            const submitterDidFfi = submitterDid ? allocCString(submitterDid) : NULL;
            rustAPI().indy_vdr_build_get_txn_request(submitterDidFfi, ledgerType, seqNo, request._handle);
            return request;
        } catch (err) {
            throw new VDRInternalError(err);
        }
    }
}
