# indy-vdr-nodejs

Wrapper for Nodejs around indy-vdr

## Requirements

This has been tested extensively with Nodejs version `16.11.0` and `16.15.0`.
Older and newer versions might also work, but they have not been tested.

## Installation

```sh
yarn add indy-vdr-nodejs
```

## Usage

You can import all types and classes from the `indy-vdr-nodejs` library:

```typescript
import { PoolCreate, GetSchemaRequest } from 'indy-vdr-nodejs'

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

> **Note**: If you want to use this library in a cross-platform environment you need to import methods from the `indy-vdr-shared` package instead. This is a platform independent package that allows to register the native bindings. The `indy-vdr-react-native` package uses this package under the hood. See the [Indy VDR Shared README](https://github.com/hyperledger/indy-vdr/tree/main/wrappers/javascript/shared/README.md) for documentation on how to use this package.
