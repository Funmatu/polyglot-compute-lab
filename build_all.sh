#!/bin/bash
set -e

echo "🔹 Building Rust (Py03)..."
maturin develop --release --features python

echo "🔹 Building Rust (WASM)..."
wasm-pack build --target web --out-dir www/pkg --no-default-features --features wasm

echo "🔹 Building Zig (WASM)..."
# Freestanding (OSなし), Dynamic libraryとしてビルド
zig build-exe zig/dll.zig -target wasm32-freestanding -O ReleaseFast -fno-entry -rdynamic -femit-bin=www/zig_dll.wasm

echo "🔹 Building WAT (WASM)..."
wat2wasm wat/dll.wat -o www/wat_dll.wasm

echo "✅ All builds finished!"