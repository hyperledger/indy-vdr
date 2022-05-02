# indy-vdr-nodejs

Wrapper for Nodejs around indy-vdr

## Requirements

This has been tested extensively with Nodejs version `16.11.0` and `16.15.0`.
Older and newer versions might also work, but they have not been tested.

## Installation

```sh
yarn add indy-vdr-nodejs indy-vdr-shared
```

## Setup

In order to work with this module a function from `indy-vdr-shared` has to be
called to register the native module (indy-vdr-nodejs)

```typescript
import { registerIndyVdr } from 'indy-vdr-shared'
import { indyVdrNodeJS } from 'indy-vdr-nodejs'

registerIndyVdr({ vdr: indyVdrNodeJS })
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
