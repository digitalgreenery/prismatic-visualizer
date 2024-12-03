#!/bin/bash


cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --target web \
    --out-dir ./wasm/ \
    --out-name "spherical_rgb_visualizer" \
    ./target/wasm32-unknown-unknown/release/spherical_rgb_visualizer.wasm
