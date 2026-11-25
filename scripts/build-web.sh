#!/usr/bin/env sh
# Build the WebAssembly engine into web/src/wasm (used by the TypeScript package in web/).
# Needs: rustup target add wasm32-unknown-unknown, and wasm-bindgen-cli at the version in Cargo.lock
#   cargo install wasm-bindgen-cli --version "$(grep -A1 'name = "wasm-bindgen"$' Cargo.lock | grep version | cut -d'"' -f2)" --locked
set -eu
cd "$(dirname "$0")/.."
cargo build --release --lib --target wasm32-unknown-unknown --features web
wasm-bindgen --target web --out-dir web/src/wasm --out-name rough_engine \
  target/wasm32-unknown-unknown/release/rough_animations.wasm
if command -v wasm-opt >/dev/null 2>&1; then
  wasm-opt -Oz -o web/src/wasm/rough_engine_bg.wasm web/src/wasm/rough_engine_bg.wasm
fi
ls -l web/src/wasm
