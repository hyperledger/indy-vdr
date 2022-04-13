import { NativeModules } from 'react-native'

const module = NativeModules.IndyVdr
const res = module.install()
if (!res) {
  throw Error('Unable to install the turboModule: indyVdr')
}

export { indyVdr } from './register'
