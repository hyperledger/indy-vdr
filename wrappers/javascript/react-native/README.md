# indy-vdr-react-native

Wrapper for React Native around indy-vdr

## Requirements

This module uses the new React Native Turbo Modules. These are faster than the
previous Native Modules, and can be completely synchronous. A React Native
version of `>= 0.66.0` is required for this package to work.

## Installation

```sh
yarn add indy-vdr-react-native indy-vdr-shared
```

## Setup

In order to work with this module a function from `indy-vdr-shared` has to be
called to register the native module (indy-vdr-react-native)

```typescript
import { registerIndyVdr } from 'indy-vdr-shared'
import { indyVdrReactNative } from 'indy-vdr-react-native'

registerIndyVdr({ vdr: indyVdrReactNative })
```

After this setup classes can be built that are imported from `indy-vdr-shared`
and afterwards be submitted as a ledger request, like so:

```typescript
const pool = new PoolCreate({
  parameters: {
    transactions: <TRANSACTION_OBJECT>
  }
})

const getSchemaRequest = new GetSchemaRequest({
  schemaId: 'J6nTnUo3YLayzc2GUUctb1:2:MyName:1.0',
})

await pool.submitRequest({ requestHandle: getSchemaRequest.handle })
```
