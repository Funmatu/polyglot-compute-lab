import initRust, { run_rust_dll, run_rust_unsafe, run_rust_bump } from './pkg/polyglot_compute_lab.js';

const ITERATIONS = 100000;

function log(msg) {
    document.getElementById('logs').textContent += msg + '\n';
    console.log(msg);
}

async function loadWasm(path) {
    const response = await fetch(path);
    const bytes = await response.arrayBuffer();
    const { instance } = await WebAssembly.instantiate(bytes, {});
    return instance.exports;
}

async function main() {
    log("Loading Modules...");
    
    // 1. Load Rust
    await initRust();
    
    // 2. Load Zig
    const zigExports = await loadWasm('./zig_dll.wasm');
    
    // 3. Load WAT
    const watExports = await loadWasm('./wat_dll.wasm');

    log("All modules loaded. Ready to benchmark.");

    // Setup Handlers
    document.getElementById('btn-rust').onclick = () => {
        const start = performance.now();
        const sum = run_rust_dll(ITERATIONS);
        const time = performance.now() - start;
        document.getElementById('res-rust').innerText = `${time.toFixed(2)} ms (Sum: ${sum})`;
    };

    // Rust (Unsafe)
    document.getElementById('btn-rust-unsafe').onclick = () => {
        log("Running Rust (Unsafe)...");
        setTimeout(() => {
            const start = performance.now();
            const sum = run_rust_unsafe(ITERATIONS);
            const time = performance.now() - start;
            document.getElementById('res-rust-unsafe').innerText = `${time.toFixed(2)} ms (Sum: ${sum})`;
        }, 10);
    };

    // Rust (Bump)
    document.getElementById('btn-rust-bump').onclick = () => {
        log("Running Rust (Bump)...");
        setTimeout(() => {
            const start = performance.now();
            const sum = run_rust_bump(ITERATIONS);
            const time = performance.now() - start;
            document.getElementById('res-rust-bump').innerText = `${time.toFixed(2)} ms (Sum: ${sum})`;
        }, 10);
    };

    document.getElementById('btn-zig').onclick = () => {
        const start = performance.now();
        const sum = zigExports.run_zig_dll(ITERATIONS);
        const time = performance.now() - start;
        document.getElementById('res-zig').innerText = `${time.toFixed(2)} ms (Sum: ${sum})`;
    };

    document.getElementById('btn-wat').onclick = () => {
        const start = performance.now();
        const sum = watExports.run_wat_dll(ITERATIONS);
        const time = performance.now() - start;
        document.getElementById('res-wat').innerText = `${time.toFixed(2)} ms (Sum: ${sum})`;
    };
}

main();