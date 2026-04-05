import type { EngineTransport, EngineTransportFactory } from '@vulfram/transport-types';
import { detectRuntime } from '@vulfram/transport-types';
import type { InitInput as WasmInitInput } from '../lib/vulfram_core.js';
import initWasmBindings, {
  vulfram_dispose,
  vulfram_get_profiling,
  vulfram_init,
  vulfram_receive_events,
  vulfram_receive_queue,
  vulfram_send_queue,
  vulfram_tick,
  vulfram_upload_buffer
} from '../lib/vulfram_core.js';

export type InitInput = WasmInitInput | Promise<WasmInitInput>;

type WasmBufferResult = {
  result: number;
  takeBuffer: () => Uint8Array;
  free: () => void;
};

type WasmBindings = {
  vulfram_dispose: () => number;
  vulfram_get_profiling: () => WasmBufferResult;
  vulfram_init: () => number;
  vulfram_receive_events: () => WasmBufferResult;
  vulfram_receive_queue: () => WasmBufferResult;
  vulfram_send_queue: (data: Uint8Array) => number;
  vulfram_tick: (timeMs: number, deltaMs: number) => number;
  vulfram_upload_buffer: (id: bigint, uploadType: number, data: Uint8Array) => number;
};

let initialized = false;
const bindings: WasmBindings = {
  vulfram_dispose,
  vulfram_get_profiling,
  vulfram_init,
  vulfram_receive_events,
  vulfram_receive_queue,
  vulfram_send_queue,
  vulfram_tick,
  vulfram_upload_buffer
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
      moduleOrPath === undefined ? '../lib/vulfram_core_bg.wasm' : 'custom-init-input';
    throw new Error(
      `Failed to initialize browser transport (runtime=${runtime.runtime}, platform=${runtime.platform ?? 'unknown'}, arch=${runtime.arch ?? 'unknown'}, expected=${expectedArtifact}): ${String(error)}`
    );
  }

  initialized = true;
}

const transportImpl: EngineTransport = {
  vulframInit: () => {
    ensureInitialized();
    return bindings.vulfram_init();
  },
  vulframDispose: () => {
    ensureInitialized();
    return bindings.vulfram_dispose();
  },
  vulframSendQueue: (buffer) => {
    ensureInitialized();
    return bindings.vulfram_send_queue(buffer);
  },
  vulframReceiveQueue: () => {
    ensureInitialized();
    return unwrapBufferResult(bindings.vulfram_receive_queue());
  },
  vulframReceiveEvents: () => {
    ensureInitialized();
    return unwrapBufferResult(bindings.vulfram_receive_events());
  },
  vulframUploadBuffer: (id, uploadType, buffer) => {
    ensureInitialized();
    return bindings.vulfram_upload_buffer(BigInt(id), uploadType, buffer);
  },
  vulframTick: (timeMs, deltaMs) => {
    ensureInitialized();
    return bindings.vulfram_tick(timeMs, deltaMs);
  },
  vulframGetProfiling: () => {
    ensureInitialized();
    return unwrapBufferResult(bindings.vulfram_get_profiling());
  }
};

export const transportBrowser: EngineTransportFactory = () => {
  ensureInitialized();
  return transportImpl;
};

export const initWasmTransport = initBrowserTransport;
export const transportWasm = transportBrowser;
