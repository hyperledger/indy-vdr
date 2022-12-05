# Indy VDR JavaScript Wrapper

This is a JavaScript wrapper around the Indy VDR library. It is intended to be used in both Node.js and React Native environments.

This document focuses on developer documentation for the wrapper. For usage see the specific readmes of the packages.

## Setup

### Prerequisites

- Node.js 16.11.0 or higher
- Yarn

### Installation

```sh
yarn install
```

### Running Tests

First make sure you've built the library using the `build.sh` script in the root of the repository.

Then run the tests with the following command. This will automatically use the local build of Indy VDR.

```sh
yarn test:local-build
```

## Releasing

You can change the version of all packages in the `wrappers/javascript` directory by running the following command:

```sh
yarn set-version 0.2.0 # make sure to change version
```

The packages will automatically be released when a release is created on GitHub, or the workflow dispatch event is triggered.
