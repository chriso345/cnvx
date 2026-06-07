# cnvx-typst

**cnvx-typst** provides Typst bindings for the [cnvx](https://github.com/chriso345/cnvx) optimization library.

> [!WARNING]
> Typst bindings are currently in early development and may not yet cover all features of the Rust library. The API is subject to change as development progresses.

---

## Installation

Requires [Rust](https://rustup.rs) and optionally [just](https://github.com/casey/just/).

`cnvx` is currently only available via source installation. Clone the repository and run the following commands to build the WebAssembly plugin from this directory:

With `just`:

```bash
just build
```

Without `just`:

```bash
cargo build --release --target wasm32-unknown-unknown

# To run the examples:
cp ../../target/wasm32-unknown-unknown/release/cnvx_typst.wasm examples/cnvx.wasm
```

You will also need the WASM target installed:

```bash
rustup target add wasm32-unknown-unknown
```

---

## License

Licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.
