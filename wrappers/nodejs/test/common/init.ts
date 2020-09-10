import { indyVdrSetDefaultLogger, initVdr } from 'src';
import { assert } from 'chai';
import { downloadGenesisFile, getNetworkGenesisFileUrl, IndyNetwork } from '../../src/tools';
import * as fs from 'fs';

export interface NetworkInfo {
    network: IndyNetwork;
    genesisFilePath: string;
}

export async function initVdrTest(): Promise<NetworkInfo> {
    const initSuccess = initVdr();
    assert.isTrue(initSuccess);
    indyVdrSetDefaultLogger();

    const selectedNetwork: string = process.env.INDY_NETWORK || 'SOVRIN_BUILDER_NET';
    const network = IndyNetwork[selectedNetwork as keyof typeof IndyNetwork];
    const genesisUrl = getNetworkGenesisFileUrl(network);
    const genesisFilePath = `${__dirname}/${selectedNetwork}.genesis.txn`;
    console.log(`genesisPath=${genesisFilePath}`);
    if (!fs.existsSync(genesisFilePath)) {
        await downloadGenesisFile(genesisUrl, genesisFilePath);
    }
    return {
        genesisFilePath,
        network,
    };
}
