// eslint-disable-next-line @typescript-eslint/no-var-requires
const { IndyVdrRequest } = require('../dist/api/indy-vdr-request');
// eslint-disable-next-line @typescript-eslint/no-var-requires
const { IndyVdrPool } = require('../dist/api/indy-vdr-pool');
// eslint-disable-next-line @typescript-eslint/no-var-requires
const { initVdr, indyVdrVersion, indyVdrSetDefaultLogger, indyVdrSetConfig } = require('../dist/api/indy-vdr-utils');
// eslint-disable-next-line @typescript-eslint/no-var-requires
const axios = require('axios');
// eslint-disable-next-line @typescript-eslint/no-var-requires
const fs = require('fs');

async function donwloadGenesis() {
    const genesisFileUrl =
        'https://raw.githubusercontent.com/sovrin-foundation/sovrin/master/sovrin/pool_transactions_builder_genesis';
    const targetFilePath = `${__dirname}/sov_buildernet.genesis.txn`;
    const { data } = await axios.get(genesisFileUrl);
    fs.writeFileSync(targetFilePath, data);
    return targetFilePath;
}

async function run() {
    const initSuccess = initVdr();
    console.log(`Init success = ${initSuccess}`);

    indyVdrSetDefaultLogger();

    const vdrVersion = indyVdrVersion();
    console.log(`vdrVersion = ${vdrVersion}`);

    const poolConfig = {
        protocol_version: 'Node1_4',
        freshness_threshold: 300,
        ack_timeout: 20,
        reply_timeout: 60,
        conn_request_limit: 5,
        conn_active_timeout: 5,
        request_read_nodes: 2,
    };
    indyVdrSetConfig(JSON.stringify(poolConfig));
    console.log(`updated vdr configuration`);

    const genesisPath = await donwloadGenesis();
    const createPoolParams = JSON.stringify({
        transactions_path: genesisPath,
    });
    const pool = IndyVdrPool.create('SovrinBuildernet', createPoolParams);
    console.log(`Created Pool ${pool.getName()} using params ${pool.getParams()}`);

    const testRequestData = {
        operation: { data: 1, ledgerId: 1, type: '3' },
        protocolVersion: 2,
        reqId: 123,
        identifier: 'LibindyDid111111111111',
    };
    const req = IndyVdrRequest.create(JSON.stringify(testRequestData));
    console.log(`Created request using params: ${req.getRequestBody()}`);

    // const res = await pool.submitRequest(req);
    // console.log(`Ledger response: ${JSON.stringify(res)}`);
}

run();
