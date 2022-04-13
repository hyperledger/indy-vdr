import { IndyVdrNativeBindings } from './turboModule'

// @ts-ignore
if (!global._indy_vdr) {
  throw Error('_indy_vdr has not been exposed on global. Something went wrong while installing the turboModule')
}

declare var _indy_vdr: IndyVdrNativeBindings

export const indyVdr = _indy_vdr
