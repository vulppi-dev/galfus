import type { EngineTransport, EngineTransportFactory } from '@galfus/transport-types';
import { detectRuntime } from '@galfus/transport-types';
import type { InitInput as WasmInitInput } from '../dist/galfus_core.js';
import initWasmBindings, {
  galfus_dispose,
  galfus_get_profiling,
  galfus_init,
  galfus_receive_events,
  galfus_receive_queue,
  galfus_send_queue,
  galfus_tick,
  galfus_upload_buffer
} from '../dist/galfus_core.js';

export type InitInput = WasmInitInput | Promise<WasmInitInput>;

type WasmBufferResult = {
  result: number;
  takeBuffer: () => Uint8Array;
  free: () => void;
};

type WasmBindings = {
  galfus_dispose: () => number;
  galfus_get_profiling: () => WasmBufferResult;
  galfus_init: () => number;
  galfus_receive_events: () => WasmBufferResult;
  galfus_receive_queue: () => WasmBufferResult;
  galfus_send_queue: (data: Uint8Array) => number;
  galfus_tick: (timeMs: number, deltaMs: number) => number;
  galfus_upload_buffer: (id: bigint, uploadType: number, data: Uint8Array) => number;
};

let initialized = false;
const bindings: WasmBindings = {
  galfus_dispose,
  galfus_get_profiling,
  galfus_init,
  galfus_receive_events,
  galfus_receive_queue,
  galfus_send_queue,
  galfus_tick,
  galfus_upload_buffer
};

function ensureInitialized(): void {
  if (!initialized) {
    throw new Error('Browser transport not initialized. Call initBrowserTransport() first.');
  }
}

function unwrapBufferResult(result: WasmBufferResult): {
  buffer: Uint8Array;
  result: number;
} {
  const buffer = result.takeBuffer();
  const code = result.result;
  result.free();
  return { buffer, result: code };
}

export async function initBrowserTransport(moduleOrPath?: InitInput): Promise<void> {
  if (initialized) return;

  try {
    await initWasmBindings(moduleOrPath);
  } catch (error) {
    const runtime = detectRuntime();
    const expectedArtifact =
      moduleOrPath === undefined ? '../dist/galfus_core_bg.wasm' : 'custom-init-input';
    throw new Error(
      `Failed to initialize browser transport (runtime=${runtime.runtime}, platform=${runtime.platform ?? 'unknown'}, arch=${runtime.arch ?? 'unknown'}, expected=${expectedArtifact}): ${String(error)}`
    );
  }

  initialized = true;
}

const transportImpl: EngineTransport = {
  galfusInit: () => {
    ensureInitialized();
    return bindings.galfus_init();
  },
  galfusDispose: () => {
    ensureInitialized();
    return bindings.galfus_dispose();
  },
  galfusSendQueue: (buffer) => {
    ensureInitialized();
    return bindings.galfus_send_queue(buffer);
  },
  galfusReceiveQueue: () => {
    ensureInitialized();
    return unwrapBufferResult(bindings.galfus_receive_queue());
  },
  galfusReceiveEvents: () => {
    ensureInitialized();
    return unwrapBufferResult(bindings.galfus_receive_events());
  },
  galfusUploadBuffer: (id, uploadType, buffer) => {
    ensureInitialized();
    return bindings.galfus_upload_buffer(BigInt(id), uploadType, buffer);
  },
  galfusTick: (timeMs, deltaMs) => {
    ensureInitialized();
    return bindings.galfus_tick(timeMs, deltaMs);
  },
  galfusGetProfiling: () => {
    ensureInitialized();
    return unwrapBufferResult(bindings.galfus_get_profiling());
  }
};

export const transportBrowser: EngineTransportFactory = () => {
  ensureInitialized();
  return transportImpl;
};

export const initWasmTransport = initBrowserTransport;
export const transportWasm = transportBrowser;
