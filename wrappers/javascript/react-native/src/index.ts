/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */

import { NativeModules } from 'react-native'

import { ReactNativeIndyVdr } from './ReactNativeIndyVdr'

const module = NativeModules.IndyVdr
if (!module.install()) throw Error('Unable to install the turboModule: indyVdr')

export * from 'indy-vdr-shared'

export const indyVdrReactNative = new ReactNativeIndyVdr()
