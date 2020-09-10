import '../../module-resolver-helper';

import { assert } from 'chai';
import { LedgerRequestGetTxn, indyVdrSetDefaultLogger, initVdr } from 'src';

describe('Pool suite', () => {
    before(async () => {
        const initSuccess = initVdr();
        assert.isTrue(initSuccess);
        indyVdrSetDefaultLogger();
    });

    describe('create:', () => {
        it.skip('should create single request instance', async () => {
            const request: LedgerRequestGetTxn = LedgerRequestGetTxn.create(0, 1, '5eDbxuffvuopHgAAAAAAAA');
            const handle: number = request.getHandle();
            assert.isNumber(handle);
        });

        it('should get request body', async () => {
            const request: LedgerRequestGetTxn = LedgerRequestGetTxn.create(0, 1, '5eDbxuffvuopHgAAAAAAAA');
            const body = request.getRequestBody();
            const bodyParsed = JSON.parse(body);
            assert.equal(bodyParsed.identifier, '5eDbxuffvuopHgAAAAAAAA');
            assert.equal(bodyParsed.operation.data, 1);
            assert.equal(bodyParsed.operation.ledgerId, 0);
            assert.equal(bodyParsed.operation.type, '3');
            assert.isNumber(bodyParsed.protocolVersion);
            assert.isNumber(bodyParsed.reqId);
        });

        // todo: enable this use case
        // it('should be possible to not specify submitter did', async () => {
        //     const request: LedgerRequestGetTxn = LedgerRequestGetTxn.create(0, 1);
        //     const body = request.getRequestBody();
        //     assert.isString(body);
        // });
    });
});
