import { indyVdrSetDefaultLogger, initVdr } from 'src';
import { assert } from 'chai';
import { downloadGenesisFile, getNetworkGenesisFileUrl, IndyNetwork } from '../../src/tools';
import * as fs from 'fs';

export async function initVdrTest(): Promise<string> {
    const initSuccess = initVdr();
    assert.isTrue(initSuccess);
    indyVdrSetDefaultLogger();

    const selectedNetwork: string = process.env.INDY_NETWORK || 'SOVRIN_BUILDER_NET';
    const network = IndyNetwork[selectedNetwork as keyof typeof IndyNetwork];
    const genesisUrl = getNetworkGenesisFileUrl(network);
    const genesisPath = `${__dirname}/${selectedNetwork}.genesis.txn`;
    console.log(`genesisPath=${genesisPath}`);
    if (!fs.existsSync(genesisPath)) {
        await downloadGenesisFile(genesisUrl, genesisPath);
    }
    return genesisPath;
}
