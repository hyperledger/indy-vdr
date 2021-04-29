import '../module-resolver-helper';

import { assert } from 'chai';
import { indyVdrSetDefaultLogger, indyVdrVersion, initVdr } from 'src';

describe('Pool:', () => {
    before(() => {
        const initSuccess = initVdr();
        assert.isTrue(initSuccess);
        indyVdrSetDefaultLogger();
    });

    describe('version', () => {
        it('should be version 0.2.1', async () => {
            const vdrVersion = indyVdrVersion();
            assert.equal(vdrVersion, '0.2.1');
        });
    });
});
