# ffi-gen

Call rust from any language. Take a look at the example to get you started.

This requires `wasm-multi-value-reverse-polyfill` to be in your path when targeting js:

```sh
cargo install --git https://github.com/vmx/wasm-multi-value-reverse-polyfill --locked
```

## Usage

You need `#![feature(vec_into_raw_parts)]` enabled on the nightly compilier on your crate, for the API bindings to work.

## Supported languages

- dart
- js

## License
Apache-2.0 OR MIT
