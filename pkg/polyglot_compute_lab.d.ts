/* tslint:disable */
/* eslint-disable */

export function run_rust_bump(iterations: number): number;

export function run_rust_dll(iterations: number): number;

export function run_rust_unsafe(iterations: number): number;

export function run_wgpu_dll(iterations: number): Promise<number>;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly run_rust_bump: (a: number) => number;
  readonly run_rust_dll: (a: number) => number;
  readonly run_rust_unsafe: (a: number) => number;
  readonly run_wgpu_dll: (a: number) => number;
  readonly __wasm_bindgen_func_elem_954: (a: number, b: number, c: number) => void;
  readonly __wasm_bindgen_func_elem_953: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_515: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_export: (a: number, b: number) => number;
  readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export3: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
