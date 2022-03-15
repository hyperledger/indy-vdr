_Generating the C header:_

Install [cbindgen](https://github.com/eqrion/cbindgen/):

```sh
cargo install cbindgen
```

From the `libindy_vdr` directory, generate the header file:

```sh
cbindgen --config include/cbindgen.toml --crate indy-vdr --output include/libindy_vdr.h
```

Note that a few types are currently defined manually, such as `ByteBuffer`, because of limitations in the binding generator.
