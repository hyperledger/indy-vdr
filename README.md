# Indy-VDR (Verifiable Data Registry)

[<img src="https://raw.githubusercontent.com/hyperledger/indy-node/master/collateral/logos/indy-logo.png" width="50%" height="auto">](https://github.com/hyperledger/indy-sdk/)

[![Unit Tests](https://github.com/hyperledger/indy-vdr/workflows/Unit%20Tests/badge.svg)](https://github.com/hyperledger/indy-vdr/actions)
[![Python Package](https://img.shields.io/pypi/v/indy-vdr.svg)](https://pypi.org/project/indy-vdr/)

- [Introduction](#introduction)
- [Features](#features)
- [Building from Source](#building-from-source)
- [Wrappers](#building-from-source)
- [Proxy Server](#proxy-server)
- [Connecting to a Ledger](#connecting-to-a-ledger)
- [How to Contribute](#how-to-contribute)

## Introduction

This library is derived from [Hyperledger Indy SDK](https://github.com/hyperledger/indy-sdk) for the more limited use case of connecting to an [Indy Node](https://github.com/hyperledger/indy-node) blockchain ledger. It is written in Rust and currently includes a Python wrapper and a standalone proxy server.

_This library is still in development and there are currently no standard release packages._

## Features

Indy-VDR can be used to connect to one or more Indy Node ledger pools given sets of genesis transactions. Methods are provided to construct ledger requests and send them to the validators, collecting the results and ensuring that there is a consensus between the nodes.

## Building from Source

First, you must have [Rust installed](https://rustup.rs/). For development, we recommend VS Code with the RLS plugin.

The library and proxy server can be built by running `cargo build` in the root directory. To build only the library, use `cargo build --lib`. You can add `--release` to produce smaller, faster binaries but with less information available for debugging purposes.

This should compile and place the shared library and `indy-vdr-proxy` executable in the `target/debug` subdirectory. The library will be named as `libindy_vdr.so` on Linux, `libindy_vdr.dll` on Windows, and `libindy_vdr.dylib` on Mac OS.

## Wrappers

The Python wrapper is located in `wrappers/python/indy_vdr`. In order for the wrapper to locate the shared library, the latter may be placed in a system shared library directory like `/usr/local/lib`. Otherwise, the location of the shared library must be added to the appropriate environment variable for your platform: `PATH` for Windows, `LD_LIBRARY_PATH` for Linux or `DYLD_LIBRARY_PATH` for Mac OS.

At a later stage it should be possible to install a precompiled 'wheel' package for your platform using `pip install indy_vdr`, but at the moment it is necessary to build the library from source.

## Proxy Server

The `indy-vdr-proxy` executable can be used to provide a simple REST API for interacting with the ledger. Command line options can be inspected by running `indy-vdr-proxy --help`.

Responses can be formatted in either HTML or JSON formats. HTML formatting is selected when the `text/html` content type is requested according to the Accept header (as sent by web browsers) or the request query string is set to `?html`. JSON formatting is selected otherwise, and may be explitly selected by using the query string `?raw`. For most ledger requests, JSON responses include information regarding which nodes were contacted is returned in the `X-Requests` header.

Sending prepared requests to the ledger is performed by delivering a POST request to the `/submit` endpoint, where the body of the request is the JSON-formatted payload. Additional endpoints are provided as shortcuts for ledger read transactions:

- `/` The root path shows basic status information about the server and the ledger pool
- `/genesis` Return the current set of genesis transactions
- `/taa` Fetch the current ledger Transaction Author Agreement
- `/aml` Fetch the current ledger Acceptance Methods List (for the TAA)
- `/nym/{DID}` Fetch the NYM transaction associated with a DID
- `/attrib/{DID}/endpoint` Fetch the registered endpoint for a DID
- `/schema/{SCHEMA_ID}` Fetch a schema by its identifier
- `/cred_def/{CRED_DEF_ID}` Fetch a credential definition by its identifier
- `/rev_reg/{REV_REG_ID}` Fetch a revocation registry by its identifier
- `/rev_reg_def/{REV_REG_ID}` Fetch a revocation registry definition by its registry identifier
- `/rev_reg_delta/{REV_REG_ID}` Fetch a revocation registry delta by its registry identifier
- `/auth` Fetch all AUTH rules for the ledger
- `/auth/{TXN_TYPE}/{ADD|EDIT}` Fetch the AUTH rule for a specific transaction type and action
- `/txn/{SUBLEDGER}/{SEQ_NO}` Fetch a specific transaction by subledger identifier (0-2, or one of `pool`, `domain`, or `config`) and sequence number.

## Connecting to a Ledger

Whether using the library or the proxy server, you will need a `genesis.txn` file containing the set of pool genesis transactions. You can run a local pool in Docker using [VON-Network](https://github.com/bcgov/von-network) or follow the [Indy-SDK instructions](https://github.com/hyperledger/indy-sdk#how-to-start-local-nodes-pool-with-docker).

However the library is used, the `RUST_LOG` environment variable may be set in order to adjust the volume of logging messages produced. Acceptable values are `error`, `warn`, `info`, `debug`, and `trace`. The `RUST_BACKTRACE` environment variable may also be set to `full` for extended output in the case of fatal errors.

## How to Contribute

- Join us on the Hyperledger Discord. Guidance at [chat.hyperledger.org](https://chat.hyperledger.org).
- Developer certificate of origin (DCO) are required in all Hyperledger repositories,
  so to get your pull requests accepted, you must certify your commits by signing off on each commit.
  More information can be found in [Signing Commits](https://github.com/hyperledger/indy-sdk/docs/contributors/signing-commits.md) article.
