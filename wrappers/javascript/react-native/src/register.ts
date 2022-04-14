import { IndyVdrNativeBindings } from 'indy-vdr-shared'

// @ts-ignore
if (!global._indy_vdr) {
  throw Error('_indy_vdr has not been exposed on global. Something went wrong while installing the turboModule')
}

declare var _indy_vdr: IndyVdrNativeBindings

export const indyVdrReactNative = _indy_vdr
