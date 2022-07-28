# Kelk examples

In this folder you can find some example contracts.
Take a look at each example to better understand how to use kelk for building your own smart contracts.

## Preparation

For building the example smart contracts you need to to add WASM32 target.

```
rustup target add wasm32-unknown-unknown
```

## Build

To build a single example and generate the contracts WASM file, navigate to the root of the smart contract and run the following command:

```
cargo +nightly build --target wasm32-unknown-unknown --release -Z unstable-options --out-dir ./wasm
```

It is recommended to remove absolute paths from the WASM binary. Check this issue for more information: https://github.com/rust-lang/rust/issues/40552

```
RUSTFLAGS="--remap-path-prefix=$(realpath ../../)=kelk --remap-path-prefix=$HOME/.cargo=cargo --remap-path-prefix=$HOME/.rustup=rustup" cargo +nightly build --target wasm32-unknown-unknown --release -Z unstable-options --out-dir ./wasm
```

You can check if the absolute paths are removed from the binary:

```
strings wasm/<example_name>.wasm | grep home
```

## Test

To test the contract you can simply run this command:

```
cargo test
```

As you can see for testing the contract, you don't need to use WASM32 target. Therefore you can use debugging tools.

## WASM optimization

Optimizing the WASM binary reduce the size of the binary file.
Download and install the latest version of [binaryen](https://github.com/WebAssembly/binaryen) first and run this command:

```
wasm-opt -Os -o wasm/<example_name>.wasm wasm/<example_name>.wasm
```
