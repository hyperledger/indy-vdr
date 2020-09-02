import { rustAPI } from '../../rustlib';
import { VDRInternalError } from '../../errors';
import { LedgerRequest } from './ledger-request';

export class LedgerRequestGetTxn extends LedgerRequest {
    constructor() {
        super();
    }

    public static create(ledgerType: number, seqNo: number, submitterDid: string): LedgerRequestGetTxn {
        try {
            const request = new LedgerRequestGetTxn();
            // TODO: IndyVDR is using "FfiStr" for submitted did. How to pass it correctly?
            // When we pass empty string "", subsequent call getRequestBody() panics
            // const submitterDidFfi = submitterDid ? Buffer.from(submitterDid) : ref.NULL_POINTER;
            rustAPI().indy_vdr_build_get_txn_request(submitterDid, ledgerType, seqNo, request._handle);
            return request;
        } catch (err) {
            throw new VDRInternalError(err);
        }
    }
}
