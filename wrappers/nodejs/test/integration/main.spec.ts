import '../module-resolver-helper';

import { assert } from 'chai';
import { IndyVdrPool, LedgerRequestCustom, LedgerRequestGetTxn } from 'src';
import { initVdrTest, NetworkInfo } from '../common/init';

describe('Integration suite', () => {
    let genesisPath: NetworkInfo;

    before(async () => {
        genesisPath = await initVdrTest();
    });

    it('should fetch transaction using custom transaction', async () => {
        const testRequestData = JSON.stringify({
            operation: { data: 1, ledgerId: 1, type: '3' },
            protocolVersion: 2,
            reqId: 1234,
            identifier: 'LibindyDid111111111111',
        });

        const request: LedgerRequestCustom = LedgerRequestCustom.create(testRequestData);
        const createPoolParams = JSON.stringify({ transactions_path: genesisPath.genesisFilePath });

        const pool: IndyVdrPool = IndyVdrPool.create(genesisPath.network.toString(), createPoolParams);
        const response = await pool.submitRequest(request);
        assert.isString(response);
        const responseObj = JSON.parse(response);
        assert.equal(responseObj.op, 'REPLY');
        assert.equal(responseObj.result.seqNo, 1);
        assert.isObject(responseObj.result.data);
        // pool.close();
    });

    it('should fetch transaction using get-txn', async () => {
        const request: LedgerRequestGetTxn = LedgerRequestGetTxn.create(1, 1, 'LibindyDid111111111111');
        const createPoolParams = JSON.stringify({ transactions_path: genesisPath.genesisFilePath });

        const pool: IndyVdrPool = IndyVdrPool.create(genesisPath.network.toString(), createPoolParams);
        const response = await pool.submitRequest(request);
        assert.isString(response);
        const responseObj = JSON.parse(response);
        assert.equal(responseObj.op, 'REPLY');
        assert.equal(responseObj.result.seqNo, 1);
        assert.isObject(responseObj.result.data);
        // pool.close();
    });

    it('should get pool status', async () => {
        const createPoolParams = JSON.stringify({ transactions_path: genesisPath.genesisFilePath });
        const pool: IndyVdrPool = IndyVdrPool.create(genesisPath.network.toString(), createPoolParams);
        const response = await pool.getStatus();
        assert.isString(response);
        const responseObj = JSON.parse(response);
        assert.isString(responseObj.mt_root);
        assert.isNumber(responseObj.mt_size);
        assert.isArray(responseObj.nodes);
        // pool.close(); // TODO: looks like VDR bug, blocks for long time and fails to close the pool
    });

    it('should close pool', async () => {
        const createPoolParams = JSON.stringify({ transactions_path: genesisPath.genesisFilePath });
        const pool: IndyVdrPool = IndyVdrPool.create(genesisPath.network.toString(), createPoolParams);
        pool.close();
        let thrown = false;
        try {
            await pool.getStatus();
        } catch (err) {
            thrown = true;
            assert.equal(err.message, 'VDR node error 2');
            assert.equal(err.vdrCode, 4); // todo: looks like VDR bug, should be 2
        }
        assert.isTrue(thrown);
    });
});
