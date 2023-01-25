import { registerIndyVdr } from '@hyperledger/indy-vdr-shared'
import { NativeModules } from 'react-native'

import { ReactNativeIndyVdr } from './ReactNativeIndyVdr'

type Module = {
  install: () => boolean
}

const module = NativeModules.IndyVdr as Module
if (!module.install()) throw Error('Unable to install the turboModule: indyVdr')

export * from '@hyperledger/indy-vdr-shared'

export const indyVdrReactNative = new ReactNativeIndyVdr()

registerIndyVdr({ vdr: indyVdrReactNative })
