import '../module-resolver-helper';

import { assert } from 'chai';
import { IndyVdrPool, IndyVdrRequest } from 'src';
import { initVdrTest } from '../common/init';

describe('Integration suite', () => {
    let genesisPath: string;

    before(async () => {
        genesisPath = await initVdrTest();
    });

    it('should fetch transaction by seqNo', async () => {
        const testRequestData = JSON.stringify({
            operation: { data: 1, ledgerId: 1, type: '3' },
            protocolVersion: 2,
            reqId: 123,
            identifier: 'LibindyDid111111111111',
        });

        const request: IndyVdrRequest = IndyVdrRequest.create(testRequestData);
        const createPoolParams = JSON.stringify({ transactions_path: genesisPath });

        const pool: IndyVdrPool = IndyVdrPool.create('Buildernet', createPoolParams);
        const response = await pool.submitRequest(request);
        assert.isString(response);
        const responseObj = JSON.parse(response);
        assert.equal(responseObj.op, 'REPLY');
        assert.equal(responseObj.result.seqNo, 1);
        assert.isObject(responseObj.result.data);
    });

    it('should get pool status', async () => {
        const createPoolParams = JSON.stringify({ transactions_path: genesisPath });
        const pool: IndyVdrPool = IndyVdrPool.create('Buildernet', createPoolParams);
        const response = await pool.getStatus();
        assert.isString(response);
        const responseObj = JSON.parse(response);
        assert.isString(responseObj.mt_root);
        assert.isNumber(responseObj.mt_size);
        assert.isArray(responseObj.nodes);
    });
});
