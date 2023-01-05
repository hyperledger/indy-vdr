import { registerIndyVdr } from 'indy-vdr-shared'

import { NodeJSIndyVdr } from './NodeJSIndyVdr'

export const indyVdrNodeJS = new NodeJSIndyVdr()
registerIndyVdr({ vdr: indyVdrNodeJS })

export * from 'indy-vdr-shared'
