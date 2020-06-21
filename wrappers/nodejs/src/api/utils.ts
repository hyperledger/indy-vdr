import { initRustAPI, rustAPI } from '../rustlib';
// import { Callback } from 'ffi-napi';
// import { createFFICallbackPromise } from '../utils/ffi-helpers';
// import { VDRInternalError } from '../errors';

export function initVdr(vdrPath?: string): boolean {
    initRustAPI(vdrPath);
    return true;
}

// export async function indyVdrPoolSubmitRequest(pool_handle: number, request_handle: number): Promise<string> {
//     try {
//         return await createFFICallbackPromise<string>(
//             (resolve, reject, cb) => {
//                 const rc = rustAPI().indy_vdr_pool_submit_request(pool_handle, request_handle, cb);
//                 if (rc) {
//                     reject(rc);
//                 }
//             },
//             (resolve, reject) =>
//                 Callback('void', ['uint32', 'uint32'], (xhandle: number, err: number, config: string) => {
//                     if (err) {
//                         reject(err);
//                         return;
//                     }
//                     resolve(config);
//                 }),
//         );
//     } catch (err) {
//         throw new VDRInternalError(err);
//     }
// }

export function indyVdrPoolCreate(params: string, pool_handle: number) {
    return rustAPI().indy_vdr_pool_create(params, pool_handle);
}

export function indyVdrVersion(): string {
    return rustAPI().indy_vdr_version();
}

export function indyVdrSetConfig(config: string): number {
    return rustAPI().indy_vdr_set_config(config);
}

export function indyVdrSetDefaultLogger(): number {
    return rustAPI().indy_vdr_set_default_logger();
}
