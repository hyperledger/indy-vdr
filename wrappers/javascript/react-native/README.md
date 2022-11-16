# indy-vdr-react-native

Wrapper for React Native around indy-vdr

## Requirements

This module uses the new React Native Turbo Modules. These are faster than the
previous Native Modules, and can be completely synchronous. A React Native
version of `>= 0.66.0` is required for this package to work.

## Installation

```sh
yarn add indy-vdr-react-native
```

## Usage

You can import all types and classes from the `indy-vdr-react-native` library:

```typescript
import { PoolCreate, GetSchemaRequest } from 'indy-vdr-react-native'

const pool = new PoolCreate({
  parameters: {
    transactions: <TRANSACTION_OBJECT>
  }
})

const getSchemaRequest = new GetSchemaRequest({
  schemaId: 'J6nTnUo3YLayzc2GUUctb1:2:MyName:1.0',
})

const schemaResponse = await pool.submitRequest(getSchemaRequest)
```

> **Note**: If you want to use this library in a cross-platform environment you need to import methods from the `indy-vdr-shared` package instead. This is a platform independent package that allows to register the native bindings. The `indy-vdr-nodejs` package uses this package under the hood. See the [Indy VDR Shared README](https://github.com/hyperledger/indy-vdr/tree/main/wrappers/javascript/shared/README.md) for documentation on how to use this package.
