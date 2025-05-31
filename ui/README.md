# Weather Data Aggregator - UI

## Development

### Toolchain Setup

```sh
cargo install trunk
cargo install leptosfmt
rustup override set 1.86.0 # only if needed
rustup target add wasm32-unknown-unknown
```

### Formatting

```sh
leptosfmt .
cargo fmt
```

### Building & Running

```
cargo build
trunk serve --open
```
