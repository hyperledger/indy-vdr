const maxTimeout = 2147483647;

export type ICbRef = Buffer;

// Lib function which accepts a callback and rejects with a return code if needed
export type ICreateFFICallbackPromiseFn<T> = (
    resolve: (value?: T) => void,
    reject: (reason?: any) => void,
    cbRef: ICbRef,
) => void;

// eslint-disable-next-line prettier/prettier
export type ICreateFFICallbackPromiseCb<T> = (
    resolve: (value?: T) => void,
    reject: (reason?: any) => void
) => ICbRef;


/**
 * Creates promise which wraps two functions "fn" and "cb". The "fn" is supposed to be responsible
 * for initiating calls into FFI. The "cb" is returns C function pointer to function which is capable
 * of handling result of the FFI call. Based on the result, it either resolves or rejects the wrapping JS promise.
 * The "fn" is also provided pointer to callback "cb" promise, which is typically provided into FFI - once the work
 * below FFI layer is done, it would call "cb".
 *
 * @param fn - function responsible for making FFI call. It needs to assure the FFI underlying code will call "cb"
 * with the result, once the work is done.
 *
 * @param cb - function returning C function pointer responsible for handling result of the FFI call
 */
export const createFFICallbackPromise = <T>(fn: ICreateFFICallbackPromiseFn<T>, cb: ICreateFFICallbackPromiseCb<T>) => {
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    let cbRef = null;
    // TODO: Research why registering a callback doesn't keep parent thread alive https://github.com/node-ffi/node-ffi
    const processKeepAliveTimer = setTimeout(() => undefined, maxTimeout);

    // Creates promise wrapping "fn" and "cb" functions, both of which can resolve or reject the promise.
    return new Promise<T>((resolve, reject) => fn(resolve, reject, (cbRef = cb(resolve, reject))))
        .then((res) => {
            cbRef = null;
            clearTimeout(processKeepAliveTimer);
            return res;
        })
        .catch((err) => {
            cbRef = null;
            clearTimeout(processKeepAliveTimer);
            throw err;
        });
};
