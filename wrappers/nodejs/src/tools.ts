import axios from 'axios';
import * as fs from 'fs';
import { URL } from 'url';

export enum IndyNetwork {
    SOVRIN_BUILDER_NET,
    SOVRIN_STAGING_NET,
    SOVRIN_MAIN_NET,
}

export function getNetworkGenesisFileUrl(network: IndyNetwork): URL {
    switch (network) {
        case IndyNetwork.SOVRIN_BUILDER_NET:
            return new URL(
                'https://raw.githubusercontent.com/sovrin-foundation/sovrin/master/sovrin/pool_transactions_builder_genesis',
            );
        case IndyNetwork.SOVRIN_STAGING_NET:
            return new URL(
                'https://raw.githubusercontent.com/sovrin-foundation/sovrin/master/sovrin/pool_transactions_sandbox_genesis',
            );
        case IndyNetwork.SOVRIN_MAIN_NET:
            return new URL(
                'https://raw.githubusercontent.com/sovrin-foundation/sovrin/master/sovrin/pool_transactions_live_genesis',
            );
        default:
            throw Error(`Unknown network ${network}`);
    }
}

export async function downloadGenesisFile(genesisFileUrl: URL, targetFilePath: string): Promise<string> {
    const { data } = await axios.get(genesisFileUrl.toString());
    fs.writeFileSync(targetFilePath, data);
    return targetFilePath;
}
