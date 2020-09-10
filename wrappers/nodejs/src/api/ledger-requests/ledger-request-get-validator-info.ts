import { rustAPI } from '../../rustlib';
import { VDRInternalError } from '../../errors';
import { LedgerRequest } from './ledger-request';
import { allocCString } from 'ref-napi';

export class LedgerRequestGetValidatorInfo extends LedgerRequest {
    constructor() {
        super();
    }

    public static create(submitterDid: string): LedgerRequestGetValidatorInfo {
        try {
            const request = new LedgerRequestGetValidatorInfo();
            rustAPI().indy_vdr_build_get_validator_info_request(allocCString(submitterDid), request._handle);
            return request;
        } catch (err) {
            throw new VDRInternalError(err);
        }
    }
}
