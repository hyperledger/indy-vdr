import * as ffi from 'ffi-napi';
import * as os from 'os';

import { FFIConfiguration, IFFIEntryPoint } from './rustlib';

export interface IVDRRuntimeConfig {
    basepath?: string;
}

const extension: Dictionary = { darwin: '.dylib', linux: '.so', win32: '.dll' };
const libPath: Dictionary = { darwin: '/usr/local/lib/', linux: '/usr/lib/', win32: 'c:\\windows\\system32\\' };

interface Dictionary {
    [Key: string]: string;
}

/**
 * @class Class which encapsulates IndyVDR FFI initialization
 */
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
        const postfix = extension[platform.toLowerCase()] || extension.linux;
        const libDir = libPath[platform.toLowerCase()] || libPath.linux;
        const library = `libindy_vdr${postfix}`;
        const customPath = process.env.LIB_INDY_VDR_PATH ? process.env.LIB_INDY_VDR_PATH + library : undefined;
        return customPath || this._config.basepath || `${libDir}${library}`;
    };
}
