[![go.dev reference](https://img.shields.io/badge/go.dev-reference-007d9c?logo=go&logoColor=white&style=for-the-badge)](https://pkg.go.dev/github.com/hyperledger/indy-vdr/wrappers/golang/vdr)

# Run demo

- The Go demo requires libindy_vdr shared library to be installed (eg: /usr/local/lib), and the header file `include/libindy_vdr.h`
  to be installed in your C include path (eg: /usr/local/include). You can run sample code using Indy VDR using command:

```
go run cmd/demo/demo.go
```

- This demo is by default a read only connection to Sovrin Buildernet network. To test writes, you
  can connect to different ledger by specifying path to genesis file and a seed for ed25519 keys that have ENDORSER or STEWARD role
  on that ledger.

```
go run cmd/demo/demo.go ~/.indy_client/pool/local/local.txn "b2352b32947e188eb72871093ac6217e"
```

- To enable indy_vdr library logs, specify `RUST_LOG` environment variable with
  desired logging level. Example:

```
RUST_LOG=trace go run cmd/demo/demo.go
```

# Requirements

- Golang >= `1.16` and CGO enabled.
