import { rustAPI } from '../../rustlib';
import { VDRInternalError } from '../../errors';
import { allocateHandleBuffer, handleBufferToNumber } from '../ffi-tools';
import * as ref from 'ref-napi';

export class LedgerRequest {
    protected _handle: Buffer;

    constructor() {
        this._handle = allocateHandleBuffer();
    }

    public getHandle(): number {
        return handleBufferToNumber(this._handle);
    }

    public getRequestBody(): string {
        try {
            const body_ptr = Buffer.alloc(8);
            // indy_vdr_request_get_body requires pointer to memory where it can store pointer to the string
            // it wants to return, essentially pointer to pointer
            rustAPI().indy_vdr_request_get_body(this.getHandle(), ref.address(body_ptr));
            // now in body_ptr is store address where the CString begins, we create pointer (buffer)
            const pointer = ref.readPointer(body_ptr, 0, 0);
            // and read the CString starting from that location, until we NULL is encountered.
            return ref.readCString(pointer, 0);
            // TODO: question is now - will GC take care of this? Or do I have to call "indy_vdr_string_free" function?
        } catch (err) {
            throw new VDRInternalError(err);
        }
    }
}
