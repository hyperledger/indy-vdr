import * as ref from 'ref-napi';
import { readUInt64LE } from 'ref-napi';

/**
 * Converts buffer handle to numeric representation
 */
export function handleBufferToNumber(handle: Buffer): number {
    return handle.readIntLE(0, 6);
}

/*
 * Handle can have at most 48bits / 6 bytes - because:
 * All numbers in Javascript are 64-bit floating point numbers, which is sufficient to represent any 48-bit integer.
 * Source: https://stackoverflow.com/questions/2575523/48-bit-bitwise-operations-in-javascript
 */
export function allocateHandleBuffer(): Buffer {
    return Buffer.alloc(6);
}

/**
 * Converts numeric handle to buffer which can be passed down to FFI layers
 */
export function handleNumberToBuffer(handle: number): Buffer {
    const buffer = allocateHandleBuffer();

    buffer.writeUIntLE(handle, 0, 6);
    return buffer;
}

/**
 * Returns address of Buffer
 */
export function getAddressOfBuffer(data: Buffer) {
    return Number(ref.address(data)).toString(16);
}

/**
 * Reads content of buffer, interprets is as little endian representation of memory address, returns it as string
 */
export function bufferTo64BitAddressString(address: Buffer) {
    const address64bit = readUInt64LE(address);
    const asString = BigInt(address64bit).toString(16); // if value > Number.MAX_SAFE_INTEGER, readUInt64LE returns the value as string
    return `0x${asString}`;
}
