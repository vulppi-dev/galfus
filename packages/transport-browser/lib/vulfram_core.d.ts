/* tslint:disable */
/* eslint-disable */

export class BufferResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    takeBuffer(): Uint8Array;
    readonly result: number;
}

/**
 * Chroma subsampling format
 */
export enum ChromaSampling {
    /**
     * Both vertically and horizontally subsampled.
     */
    Cs420 = 0,
    /**
     * Horizontally subsampled.
     */
    Cs422 = 1,
    /**
     * Not subsampled.
     */
    Cs444 = 2,
    /**
     * Monochrome.
     */
    Cs400 = 3,
}

export function vulfram_dispose(): number;

export function vulfram_get_profiling(): BufferResult;

export function vulfram_init(): number;

export function vulfram_receive_events(): BufferResult;

export function vulfram_receive_queue(): BufferResult;

export function vulfram_send_queue(data: Uint8Array): number;

export function vulfram_tick(time_ms: number, delta_ms: number): number;

export function vulfram_upload_buffer(id: bigint, upload_type: number, data: Uint8Array): number;

export function wasm_start(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_bufferresult_free: (a: number, b: number) => void;
    readonly bufferresult_result: (a: number) => number;
    readonly bufferresult_takeBuffer: (a: number, b: number) => void;
    readonly vulfram_dispose: () => number;
    readonly vulfram_get_profiling: () => number;
    readonly vulfram_init: () => number;
    readonly vulfram_receive_events: () => number;
    readonly vulfram_receive_queue: () => number;
    readonly vulfram_send_queue: (a: number, b: number) => number;
    readonly vulfram_tick: (a: number, b: number) => number;
    readonly vulfram_upload_buffer: (a: bigint, b: number, c: number, d: number) => number;
    readonly wasm_start: () => void;
    readonly __wasm_bindgen_func_elem_2090: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_2092: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_4573: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_2091: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_2091_3: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_start: () => void;
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
