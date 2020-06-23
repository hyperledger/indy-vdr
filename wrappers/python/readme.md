# Run demo
- You can ran sample code using Indy VDR using command:
```
python -m demo.test
```

- This demo is by default connection to Sovrin Buildernet network but you
can connect to different ledger by specifying path to genesis file. For example:
```
python -m demo.test ~/.indy_client/pool/local/local.txn
```

- To enable indy_vdr library logs, specify `RUST_LOG` environment variable with
desired logging level. Example:
```
RUST_LOG=trace python -m demo.test
```

# Requirements
- Python of version `3.6.3` or higher.