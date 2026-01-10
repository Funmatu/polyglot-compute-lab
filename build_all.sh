#!/bin/bash
set -e

# -C target-cpu=native : 現在のCPUに最適な命令セット（AVX2など）を使用する
export RUSTFLAGS="-C target-cpu=native"

echo "🔹 Building Rust (Py03)..."
maturin develop --release --features python

echo "🔹 Building Rust (WASM)..."
wasm-pack build --target web --out-dir www/pkg --no-default-features --features wasm

echo "🔹 Building Zig (WASM)..."
# Freestanding (OSなし), Dynamic libraryとしてビルド
zig build-exe zig/dll.zig -target wasm32-freestanding -O ReleaseFast -fno-entry -rdynamic -femit-bin=www/zig_dll.wasm

echo "🔹 Building Zig Zipper (WASM)..."
zig build-exe zig/zipper.zig -target wasm32-freestanding -O ReleaseFast -fno-entry -rdynamic -femit-bin=www/zig_zipper.wasm

echo "🔹 Building WAT (WASM)..."
wat2wasm wat/dll.wat -o www/wat_dll.wasm

echo "🔹 Building Zig Zipper (Native Shared Library)..."
# -dynamic: 共有ライブラリを作成
# -O ReleaseFast: 最適化全開
zig build-lib zig/zipper.zig -dynamic -O ReleaseFast -femit-bin=zig_zipper.so

echo "✅ All builds finished!"