import { VDRInternalError } from '../errors';

export const errorMessage = (errorCode: number | Error): string => {
    if (errorCode instanceof VDRInternalError) {
        return errorCode.message;
    }
    if (errorCode instanceof Error) {
        const message = 'VDR node error 1';
        return `${message}: ${errorCode.message}`;
    }
    return 'VDR node error 2';
};
