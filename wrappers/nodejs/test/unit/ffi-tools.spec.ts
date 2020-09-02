import '../module-resolver-helper';
import { bufferTo64BitAddressString, handleBufferToNumber, handleNumberToBuffer } from '../../src/api/ffi-tools';
import { assert } from 'chai';

describe('Common tools suite', () => {
    describe('buffer conversions', () => {
        it('should convert buffer to number', async () => {
            {
                const data1 = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
                const buf1 = Buffer.from(data1);
                const num1 = handleBufferToNumber(buf1);
                assert.equal(num1, 1);
            }
            {
                const data2 = [0x00, 0x01, 0x00, 0x00, 0x00, 0x00];
                const buf2 = Buffer.from(data2);
                const num2 = handleBufferToNumber(buf2);
                assert.equal(num2, 256);
            }
            {
                const data1 = [0xff, 0xff, 0xff, 0xff, 0xff, 0x0f];
                const buf1 = Buffer.from(data1);
                const num1 = handleBufferToNumber(buf1);
                assert.equal(num1, 17592186044415);
            }
            {
                const data1 = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
                const buf1 = Buffer.from(data1);
                const num1 = handleBufferToNumber(buf1);
                assert.equal(num1, -1);
            }
        });

        it('should convert number to buffer', async () => {
            const buf1 = handleNumberToBuffer(0);
            assert.equal(buf1.length, 6);
            assert.equal(buf1[0], 0);
            assert.equal(buf1[1], 0);
            assert.equal(buf1[2], 0);
            assert.equal(buf1[3], 0);
            assert.equal(buf1[4], 0);
            assert.equal(buf1[5], 0);

            const buf2 = handleNumberToBuffer(256);
            assert.equal(buf2.length, 6);
            assert.equal(buf2[0], 0);
            assert.equal(buf2[1], 1);
            assert.equal(buf2[2], 0);
            assert.equal(buf2[3], 0);
            assert.equal(buf2[4], 0);
            assert.equal(buf2[5], 0);
        });

        it('should read buffer as little endian', async () => {
            const address = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
            const aaddresInBuffer = Buffer.from(address);
            const hexAddressString = bufferTo64BitAddressString(aaddresInBuffer);
            assert.equal(hexAddressString, '0x807060504030201');
        });
    });
});
