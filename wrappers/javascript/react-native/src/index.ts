import { NativeModules } from 'react-native'

import { ReactNativeIndyVdr } from './indyVdr'

const module = NativeModules.IndyVdr
const res = module.install()
if (!res) {
  throw Error('Unable to install the turboModule: indyVdr')
}

export const indyVdrReactNative = new ReactNativeIndyVdr()
