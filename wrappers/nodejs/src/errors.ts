import { errorMessage } from './utils/error-message';

export class VDRInternalError extends Error {
    public readonly vdrCode: number;
    public readonly inheritedStackTraces: any[] = [];

    constructor(code: number | Error) {
        super(errorMessage(code));
        if (code instanceof Error) {
            if (code.stack) {
                this.inheritedStackTraces.push(code.stack);
            }
            if (code instanceof VDRInternalError) {
                this.vdrCode = code.vdrCode;
                this.inheritedStackTraces.unshift(...code.inheritedStackTraces);
                return this;
            }
            this.vdrCode = 1234;
            return this;
        }
        this.vdrCode = code;
    }
}
