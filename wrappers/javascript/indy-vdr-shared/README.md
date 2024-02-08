# Indy VDR Shared

This package does not contain any functionality, just the classes and types
that wrap around the native NodeJS / React Native functionality

## General setup

Every function can be called by creating an instance of a class with the
correct parameters. This class will have a `handle` as a property and that can
be used to create a ledger request. Following is a small example to request a
schema from a ledger.

```typescript
import { PoolCreate, GetSchemaRequest } from '@hyperledger/indy-vdr-shared'

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

## Platform independent setup

If you would like to leverage the Indy VDR libraries for JavaScript in a platform independent way you need to add the `@hyperledger/indy-vdr-shared` package to your project. This package exports all public methods.

Before calling any methods you then need to make sure you register the platform specific native bindings. You can do this by importing the platform specific package. You can do this by having separate files that register the package, which allows the React Native bundler to import a different package:

```typescript
// register.ts
import '@hyperledger/indy-vdr-nodejs'
```

```typescript
// register.native.ts
import '@hyperledger/indy-vdr-react-native'
```

An alterative approach is to first try to require the Node.JS package, and otherwise require the React Native package:

```typescript
try {
  require('@hyperledger/indy-vdr-nodejs')
} catch (error) {
  try {
    require('@hyperledger/indy-vdr-react-native')
  } catch (error) {
    throw new Error('Could not load Indy VDR bindings')
  }
}
```

How you approach it is up to you, as long as the native binding are called before any actions are performed on the Indy VDR library.

## Version Compatibility

The JavaScript wrapper is versioned independently from the native bindings. The following table shows the compatibility between the different versions:

| Indy VDR      | JavaScript Wrapper |
| ------------- | ------------------ |
| v0.4.0-dev.16 | v0.1.0             |
| v0.4.1        | v0.2.0             |
