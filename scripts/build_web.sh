#!/usr/bin/env bash
set -euo pipefail
cargo build --locked --release --target wasm32-unknown-unknown --example playground --features playground
mkdir -p dist/pkg
wasm-bindgen --target web --remove-name-section --out-dir dist/pkg --no-typescript target/wasm32-unknown-unknown/release/examples/playground.wasm
cp web/index.html web/style.css web/app.js dist/
cp assets/icon.svg dist/icon.svg
