import axios from 'axios';
import * as fs from 'fs';

export async function donwloadGenesis(): Promise<string> {
    const genesisFileUrl =
        'https://raw.githubusercontent.com/sovrin-foundation/sovrin/master/sovrin/pool_transactions_builder_genesis';
    const targetFilePath = `${__dirname}/sov_buildernet.genesis.txn`;
    const { data } = await axios.get(genesisFileUrl);
    fs.writeFileSync(targetFilePath, data);
    return targetFilePath;
}
