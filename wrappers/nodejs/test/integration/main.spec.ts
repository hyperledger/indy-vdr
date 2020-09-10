import '../module-resolver-helper';

import { assert } from 'chai';
import { IndyVdrPool, LedgerRequestCustom, LedgerRequestGetTxn } from 'src';
import { initVdrTest, NetworkInfo } from '../common/init';
import { LedgerRequestNym } from '../../src/api/ledger-requests/ledger-request-nym';
import { LedgerRequestGetValidatorInfo } from '../../src/api/ledger-requests/ledger-request-get-validator-info';
import { IndyNetwork } from '../../src/tools';
import { LedgerRequestGetNym } from '../../src/api/ledger-requests/ledger-request-get-nym';
import {LedgerRequestGetSchema} from "../../src/api/ledger-requests/ledger-request-get-schema";

describe('Integration suite', () => {
    let network: NetworkInfo;

    before(async () => {
        network = await initVdrTest();
    });

    it('should fetch transaction using custom transaction', async () => {
        const testRequestData = JSON.stringify({
            operation: { data: 1, ledgerId: 1, type: '3' },
            protocolVersion: 2,
            reqId: 1234,
            identifier: 'LibindyDid111111111111',
        });

        const request: LedgerRequestCustom = LedgerRequestCustom.create(testRequestData);
        const createPoolParams = JSON.stringify({ transactions_path: network.genesisFilePath });

        const pool: IndyVdrPool = IndyVdrPool.create(network.network.toString(), createPoolParams);
        const response = await pool.submitRequest(request);
        assert.isString(response);
        const responseObj = JSON.parse(response);
        assert.equal(responseObj.op, 'REPLY');
        assert.equal(responseObj.result.seqNo, 1);
        assert.isObject(responseObj.result.data);
        // pool.close();
    });

    it('should fetch transaction using get-txn', async () => {
        const request: LedgerRequestGetTxn = LedgerRequestGetTxn.create(1, 1);
        const createPoolParams = JSON.stringify({ transactions_path: network.genesisFilePath });

        const pool: IndyVdrPool = IndyVdrPool.create(network.network.toString(), createPoolParams);
        const response = await pool.submitRequest(request);
        assert.isString(response);
        const responseObj = JSON.parse(response);
        assert.equal(responseObj.op, 'REPLY');
        assert.equal(responseObj.result.seqNo, 1);
        assert.isObject(responseObj.result.data);
        // pool.close();
    });

    // todo: to test this, we'd need to sign the request and attach it using indy_vdr_request_set_signature
    it.skip('should fetch transaction using nym', async () => {
        const request: LedgerRequestGetTxn = LedgerRequestNym.create(
            'LibindyDid111111111111',
            'FbjuFFq6jLsSMdgN9ifErE',
        );
        const createPoolParams = JSON.stringify({ transactions_path: network.genesisFilePath });

        const pool: IndyVdrPool = IndyVdrPool.create(network.network.toString(), createPoolParams);
        const response = await pool.submitRequest(request);
        assert.isString(response);
        console.log(JSON.stringify(JSON.parse(response), null, 2));
        // pool.close();
    });

    // todo: to test this, we'd need to sign the request and attach it using indy_vdr_request_set_signature
    it.skip('should get validator info', async () => {
        const request: LedgerRequestGetValidatorInfo = LedgerRequestGetValidatorInfo.create('FbjuFFq6jLsSMdgN9ifErE');
        const createPoolParams = JSON.stringify({ transactions_path: network.genesisFilePath });

        const pool: IndyVdrPool = IndyVdrPool.create(network.network.toString(), createPoolParams);
        const response = await pool.submitRequest(request);
        assert.isString(response);
        console.log(JSON.stringify(JSON.parse(response), null, 2));
    });

    function getExistingDidForNetwork(network: IndyNetwork): string {
        if (network === IndyNetwork.SOVRIN_MAIN_NET) {
            return '4nbERyUuQuEGDxmBZqisda';
        } else if (network === IndyNetwork.SOVRIN_STAGING_NET) {
            return 'VCAi7DaxdTAJAv2uQpuA8B';
        } else if (network === IndyNetwork.SOVRIN_BUILDER_NET) {
            return 'V5qJo72nMeF7x3ci8Zv2WP';
        }
        throw Error(`Unknown network`);
    }

    it('should get nym', async () => {
        const queryDid = getExistingDidForNetwork(network.network);
        const request: LedgerRequestGetNym = LedgerRequestGetNym.create(queryDid);
        const createPoolParams = JSON.stringify({ transactions_path: network.genesisFilePath });

        const pool: IndyVdrPool = IndyVdrPool.create(network.network.toString(), createPoolParams);
        const response = await pool.submitRequest(request);
        const responseObj = JSON.parse(response);
        assert.equal(responseObj.op, 'REPLY');
        assert.isNumber(responseObj.result.seqNo);
        assert.isString(responseObj.result.data);
    });

    function getSchemaIdForNetwork(network: IndyNetwork): string {
        if (network === IndyNetwork.SOVRIN_MAIN_NET) {
            return '4xE68b6S5VRFrKMMG1U95M:2:Practicing Certificate:1.0.0';
        } else if (network === IndyNetwork.SOVRIN_STAGING_NET) {
            return 'ALyqhiVkmT2zDLdNvBNZzm:2:SchemaExampleSAP:1.1';
        } else if (network === IndyNetwork.SOVRIN_BUILDER_NET) {
            return 'FbjuFFq6jLsSMdgN9ifErE:2:Specialitate Medic:1.0';
        }
        throw Error(`Unknown network`);
    }

    it('should get schema', async () => {
        const querySchemaId = getSchemaIdForNetwork(network.network);
        const request: LedgerRequestGetSchema = LedgerRequestGetSchema.create(querySchemaId);
        const createPoolParams = JSON.stringify({ transactions_path: network.genesisFilePath });

        const pool: IndyVdrPool = IndyVdrPool.create(network.network.toString(), createPoolParams);
        const response = await pool.submitRequest(request);
        const responseObj = JSON.parse(response);
        assert.equal(responseObj.op, 'REPLY');
        assert.isNumber(responseObj.result.seqNo);
        assert.isObject(responseObj.result.state_proof);
        assert.isArray(responseObj.result.data.attr_names);
    });

    it('should get pool status', async () => {
        const createPoolParams = JSON.stringify({ transactions_path: network.genesisFilePath });
        const pool: IndyVdrPool = IndyVdrPool.create(network.network.toString(), createPoolParams);
        const response = await pool.getStatus();
        assert.isString(response);
        const responseObj = JSON.parse(response);
        assert.isString(responseObj.mt_root);
        assert.isNumber(responseObj.mt_size);
        assert.isArray(responseObj.nodes);
        // pool.close(); // TODO: looks like VDR bug, blocks for long time and fails to close the pool
    });

    it('should close pool', async () => {
        const createPoolParams = JSON.stringify({ transactions_path: network.genesisFilePath });
        const pool: IndyVdrPool = IndyVdrPool.create(network.network.toString(), createPoolParams);
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
