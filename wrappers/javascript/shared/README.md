# indy-vdr-shared

This package does not contain any functionality, just the classes and types
that wrap around the native NodeJS / React Native functionality

## General setup

Every function can be called by creating an instance of a class with the
correct parameters. This class will have a `handle` as a property and that can
be used to create a ledger request. Following is a small example to request a
schema from a ledger.

```typescript
const pool = new PoolCreate({
  parameters: {
    transactions: <TRANSACTION_OBJECT>
  }
})

const getSchemaRequest = new GetSchemaRequest({
  schemaId: 'J6nTnUo3YLayzc2GUUctb1:2:MyName:1.0',
})

const schemaObject =
  await pool.submitRequest({ requestHandle: getSchemaRequest.handle })
```
