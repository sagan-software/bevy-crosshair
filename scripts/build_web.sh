#!/usr/bin/env bash
set -euo pipefail
cargo build --locked --release --target wasm32-unknown-unknown --example playground --features playground
web_output="${DIST_DIR:-dist}"
mkdir -p "$web_output/pkg"
wasm-bindgen --target web --remove-name-section --out-dir "$web_output/pkg" --no-typescript target/wasm32-unknown-unknown/release/examples/playground.wasm
cp web/index.html web/style.css web/app.js web/presets.js "$web_output/"
cp assets/icon.svg "$web_output/icon.svg"
