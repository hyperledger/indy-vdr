import * as ffi from 'ffi-napi';
import * as os from 'os';

import { FFIConfiguration, IFFIEntryPoint } from './rustlib';

export interface IVDRRuntimeConfig {
    basepath?: string;
}

// VDRRuntime is the object that interfaces with the indy vdr functions
// FFIConfiguration contain sdk api functions
// VDRuntimeConfig is a class to enable explicit specification of indy vdr library file

const extension = { darwin: '.dylib', linux: '.so', win32: '.dll' };
const libPath = { darwin: '/usr/local/lib/', linux: '/usr/lib/', win32: 'c:\\windows\\system32\\' };

export class VDRRuntime {
    public readonly ffi: IFFIEntryPoint;
    private _config: IVDRRuntimeConfig;

    constructor(config: IVDRRuntimeConfig = {}) {
        this._config = config;
        // initialize FFI
        const libraryPath = this._initializeBasepath();
        this.ffi = ffi.Library(libraryPath, FFIConfiguration);
    }

    private _initializeBasepath = (): string => {
        const platform = os.platform();
        // @ts-ignore
        const postfix = extension[platform.toLowerCase()] || extension.linux;
        // @ts-ignore
        const libDir = libPath[platform.toLowerCase()] || libPath.linux;
        const library = `libindy_vdr${postfix}`;
        const customPath = process.env.LIB_INDY_VDR_PATH ? process.env.LIB_INDY_VDR_PATH + library : undefined;
        return customPath || this._config.basepath || `${libDir}${library}`;
    };
}
