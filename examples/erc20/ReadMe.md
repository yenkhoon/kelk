## Token contract

A simple ERC-20 token contractor.

## How to compile


To compile the contract as WASM binary, first you need to add WASM32 target.
```
rustup target add wasm32-unknown-unknown
```

Then you can compile it like this:
```
cargo +nightly build --target wasm32-unknown-unknown --release -Z unstable-options --out-dir ./wasm
```

It is recommended to remove absolute paths from the WASM binary. Check this issue for more information: https://github.com/rust-lang/rust/issues/40552
```
RUSTFLAGS="--remap-path-prefix=$(realpath ../../)=kelk --remap-path-prefix=$HOME/.cargo=cargo --remap-path-prefix=$HOME/.rustup=rustup" cargo +nightly build --target wasm32-unknown-unknown --release -Z unstable-options --out-dir ./wasm
```

You can check if the absolute paths are removed from the binary:

```
strings wasm/helo_world.wasm | grep home
```

## How to test

To test the contract you can simply run this command:

```
cargo +nightly test
```

As you can see to test the contract you don't need to use WASM32 target. Therefore you can use debugging tools.

## WASM optimization

Optimizing the WASM binary reduce the size of the binary file. Download and install the latest version of [binaryen](https://github.com/WebAssembly/binaryen) first.

```
wasm-opt -Os -o wasm/calculator.wasm wasm/calculator.wasm
```
