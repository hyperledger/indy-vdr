import '../module-resolver-helper';

import { assert } from 'chai';
import { IndyVdrPool } from 'src';
import { initVdrTest, NetworkInfo } from '../common/init';

describe('Pool suite', () => {
    let genesisPath: NetworkInfo;

    before(async () => {
        genesisPath = await initVdrTest();
    });

    describe('create:', () => {
        it('many pool instances with unique handles', async () => {
            const createPoolParams = JSON.stringify({ transactions_path: genesisPath.genesisFilePath });
            const poolHandles = [];
            for (let i = 1; i < 20; i++) {
                const pool: IndyVdrPool = IndyVdrPool.create('pool_foo', createPoolParams);
                const poolHandle: number = pool.getHandle();
                assert.equal(poolHandle, i);
                poolHandles.push(poolHandle);
            }
        });

        it('success', async () => {
            const createPoolParams = JSON.stringify({ transactions_path: genesisPath.genesisFilePath });

            const pool: IndyVdrPool = IndyVdrPool.create('pool_foo', createPoolParams);
            const poolHandle: number = pool.getHandle();
            assert.isNumber(poolHandle);
            assert.equal(pool.getName(), 'pool_foo');
            assert.equal(pool.getParams(), createPoolParams);

            const pool2: IndyVdrPool = IndyVdrPool.create('pool_bar', createPoolParams);
            const poolHandle2: number = pool2.getHandle();
            assert.isNumber(poolHandle2);
            assert.equal(pool2.getName(), 'pool_bar');
            assert.equal(pool2.getParams(), createPoolParams);

            assert.notEqual(poolHandle, poolHandle2);
        });

        it('should get pool transactions', async () => {
            const createPoolParams = JSON.stringify({ transactions_path: genesisPath.genesisFilePath });
            const pool: IndyVdrPool = IndyVdrPool.create('foo', createPoolParams);
            const response = await pool.getPoolTransactions();
            const lines = response.split('\n');
            for (let i = 0; i < lines.length; i++) {
                const poolTx = JSON.parse(lines[i]);
                assert.isObject(poolTx.txn);
                assert.isObject(poolTx.txnMetadata);
                assert.isString(poolTx.ver);
            }
        });

        // Todo: I suppose rust should check the path is valid and throw error if not
        // it('bad path', async () => {
        //     const createPoolParams = JSON.stringify({ transactions_path: '/tmp/foo/bar/42' });
        //     const pool: IndyVdrPool = IndyVdrPool.create(createPoolParams);
        // });
    });
});
