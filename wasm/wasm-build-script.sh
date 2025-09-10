#!/bin/bash

export RUSTFLAGS=--cfg=web_sys_unstable_apis
export RUSTFLAGS='--cfg getrandom_backend="wasm_js"'
cargo clean
cargo build --release --target wasm32-unknown-unknown || exit 1
wasm-bindgen --no-typescript --target web \
    --out-dir ./wasm/ \
    --out-name "prismatic_visualizer" \
    ./target/wasm32-unknown-unknown/release/prismatic_visualizer.wasm || exit 1

wasm-opt -Oz -o wasm/prismatic_visualizer_bg.wasm wasm/prismatic_visualizer_bg.wasm || exit 1

echo "Build and optimization completed successfully."
