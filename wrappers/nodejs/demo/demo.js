// eslint-disable-next-line @typescript-eslint/no-var-requires
const { IndyVdrPool, LedgerRequestCustom, LedgerRequestGetTxn } = require('../dist');
// eslint-disable-next-line @typescript-eslint/no-var-requires
const { initVdr, indyVdrVersion, indyVdrSetDefaultLogger, indyVdrSetConfig } = require('../dist/api/indy-vdr-utils');
// eslint-disable-next-line @typescript-eslint/no-var-requires
const axios = require('axios');
// eslint-disable-next-line @typescript-eslint/no-var-requires
const fs = require('fs');
// eslint-disable-next-line @typescript-eslint/no-var-requires
const logger = require('./logger')('demo');

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
    logger.info(`Init success = ${initSuccess}`);

    indyVdrSetDefaultLogger();

    const vdrVersion = indyVdrVersion();
    logger.info(`vdrVersion = ${vdrVersion}`);

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
    logger.info(`updated vdr configuration`);

    const genesisPath = await donwloadGenesis();
    const createPoolParams = JSON.stringify({
        transactions_path: genesisPath,
    });
    const pool = IndyVdrPool.create('SovrinBuildernet', createPoolParams);
    logger.info(`Created Pool ${pool.getName()} using params ${pool.getParams()}`);

    const testRequestData = {
        operation: { data: 1, ledgerId: 1, type: '3' },
        protocolVersion: 2,
        reqId: 123,
        identifier: 'LibindyDid111111111111',
    };
    {
        const req = LedgerRequestCustom.create(JSON.stringify(testRequestData));
        logger.debug(`Created custom request: ${req.getRequestBody()}`);
        const res = await pool.submitRequest(req);
        logger.info(`Ledger response: ${JSON.stringify(JSON.parse(res), null, 2)}`);
    }
    {
        const req = LedgerRequestGetTxn.create(1, 1, 'LibindyDid111111111111');
        logger.debug(`Created get-txn request: ${req.getRequestBody()}`);
        const res = await pool.submitRequest(req);
        logger.info(`Ledger response: ${JSON.stringify(JSON.parse(res), null, 2)}`);
    }
}

run();
