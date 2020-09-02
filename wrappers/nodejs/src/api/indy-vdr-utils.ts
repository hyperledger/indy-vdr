import { initRustAPI, rustAPI } from '../rustlib';

export function initVdr(vdrPath?: string): boolean {
    initRustAPI(vdrPath);
    return true;
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
