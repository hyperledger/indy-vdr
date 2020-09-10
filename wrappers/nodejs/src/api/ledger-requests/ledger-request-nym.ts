import { rustAPI } from '../../rustlib';
import { VDRInternalError } from '../../errors';
import { LedgerRequest } from './ledger-request';
import { allocCString, NULL } from 'ref-napi';

/**
 * Builds NYM transaction. In order to submit this successfully, you'll have to attach signature using
 * indy_vdr_request_set_signature or indy_vdr_request_add_multi_signature
 */
export class LedgerRequestNym extends LedgerRequest {
    constructor() {
        super();
    }

    public static create(
        submitterDid: string,
        dest: string,
        verkey?: string,
        alias?: string,
        role?: string,
    ): LedgerRequestNym {
        try {
            const request = new LedgerRequestNym();
            const verkeyFfi = verkey ? allocCString(verkey) : NULL;
            const aliasFfi = alias ? allocCString(alias) : NULL;
            const roleFfi = role ? allocCString(role) : NULL;
            rustAPI().indy_vdr_build_nym_request(
                allocCString(submitterDid),
                allocCString(dest),
                verkeyFfi,
                aliasFfi,
                roleFfi,
                request._handle,
            );
            return request;
        } catch (err) {
            throw new VDRInternalError(err);
        }
    }
}
