// eslint-disable-next-line @typescript-eslint/no-var-requires
const { initVdr, indyVdrVersion, indyVdrSetDefaultLogger, indyVdrSetConfig } = require('../dist/api/utils');

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
}

run();
