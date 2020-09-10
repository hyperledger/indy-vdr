import '../../module-resolver-helper';

import { assert } from 'chai';
import { LedgerRequestCustom, indyVdrSetDefaultLogger, initVdr } from 'src';

describe('Pool suite', () => {
    before(async () => {
        const initSuccess = initVdr();
        assert.isTrue(initSuccess);
        indyVdrSetDefaultLogger();
    });

    const testRequestData = JSON.stringify({
        operation: { data: 1, ledgerId: 1, type: '3' },
        protocolVersion: 2,
        reqId: 123,
        identifier: 'LibindyDid111111111111',
    });

    describe('create:', () => {
        it('should create single request instance', async () => {
            const request: LedgerRequestCustom = LedgerRequestCustom.create(testRequestData);
            const handle: number = request.getHandle();
            assert.isNumber(handle);
        });

        it('request handles should be increment pre each new request created', async () => {
            const handles = [];
            for (let i = 1; i < 19; i++) {
                const request: LedgerRequestCustom = LedgerRequestCustom.create(testRequestData);
                const handle: number = request.getHandle();
                if (handles.length > 0) {
                    assert.equal(handle, handles[handles.length - 1] + 1);
                }
                handles.push(handle);
            }
        });

        it('should get request body', async () => {
            const request: LedgerRequestCustom = LedgerRequestCustom.create(testRequestData);
            const body = request.getRequestBody();
            assert.deepEqual(JSON.parse(body), JSON.parse(testRequestData));
        });

        // Todo: I suppose rust should check the path is valid and throw error if not
        // it('bad path', async () => {
        //     const createPoolParams = JSON.stringify({ transactions_path: '/tmp/foo/bar/42' });
        //     const pool: IndyVdrPool = IndyVdrPool.create(createPoolParams);
        // });
    });
});
