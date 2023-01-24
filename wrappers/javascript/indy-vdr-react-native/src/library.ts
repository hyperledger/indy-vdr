import type { IndyVdrNativeBindings } from '@hyperledger/indy-vdr-shared'

// This can already check whether `_indy_vdr` exists on global
// eslint-disable-next-line @typescript-eslint/no-use-before-define
if (!_indy_vdr) {
  throw Error('_indy_vdr has not been exposed on global. Something went wrong while installing the turboModule')
}

declare let _indy_vdr: IndyVdrNativeBindings

export const indyVdrReactNative = _indy_vdr
