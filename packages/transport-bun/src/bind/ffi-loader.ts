import { dlopen, ptr, toArrayBuffer, type Pointer } from 'bun:ffi';
import {
  detectRuntime,
  getArtifactFileName,
  resolveNativePlatform,
  selectPlatformLoader,
  type PlatformLoaderMap,
} from '@vulfram/transport-types';
import type { BufferResult } from './types';

const loaders: PlatformLoaderMap<{ default: string }> = {
  darwin: {
    arm64: () =>
      // @ts-expect-error
      import('../../lib/macos-arm64/vulfram_core.dylib', {
        with: { type: 'file' },
      }),
    x64: () =>
      // @ts-expect-error
      import('../../lib/macos-x64/vulfram_core.dylib', {
        with: { type: 'file' },
      }),
  },
  linux: {
    arm64: () =>
      // @ts-expect-error
      import('../../lib/linux-arm64/vulfram_core.so', {
        with: { type: 'file' },
      }),
    x64: () =>
      // @ts-expect-error
      import('../../lib/linux-x64/vulfram_core.so', {
        with: { type: 'file' },
      }),
  },
  win32: {
    arm64: () =>
      // @ts-expect-error
      import('../../lib/windows-arm64/vulfram_core.dll', {
        with: { type: 'file' },
      }),
    x64: () =>
      // @ts-expect-error
      import('../../lib/windows-x64/vulfram_core.dll', {
        with: { type: 'file' },
      }),
  },
};

function getExpectedLocalArtifact(): string {
  try {
    const platform = resolveNativePlatform();
    const filename = getArtifactFileName('ffi', platform);
    return `../../lib/${platform}/${filename}`;
  } catch {
    return '../../lib/<platform>/vulfram_core.<dll|dylib|so>';
  }
}

async function resolveNativeLibraryPath(): Promise<string> {
  const importLoader = selectPlatformLoader(loaders, 'FFI');

  try {
    return (await importLoader()).default;
  } catch (error) {
    const runtime = detectRuntime();
    const expectedArtifact = getExpectedLocalArtifact();
    throw new Error(
      `Failed to load bundled FFI artifact (runtime=${runtime.runtime}, platform=${runtime.platform ?? 'unknown'}, arch=${runtime.arch ?? 'unknown'}, expected=${expectedArtifact}): ${String(error)}`,
    );
  }
}

const lib = await resolveNativeLibraryPath();

const { symbols: VULFRAM_CORE_DYLIB, close } = dlopen(lib, {
  vulfram_init: { args: [], returns: 'u32' },
  vulfram_dispose: { args: [], returns: 'u32' },
  vulfram_send_queue: { args: ['ptr', 'usize'], returns: 'u32' },
  vulfram_receive_queue: { args: ['ptr', 'ptr'], returns: 'u32' },
  vulfram_receive_events: { args: ['ptr', 'ptr'], returns: 'u32' },
  vulfram_upload_buffer: {
    args: ['u64', 'u32', 'ptr', 'usize'],
    returns: 'u32',
  },
  vulfram_tick: { args: ['u64', 'u32'], returns: 'u32' },
  vulfram_get_profiling: { args: ['ptr', 'ptr'], returns: 'u32' },
});

process.once('beforeExit', () => {
  close();
});

function vulframDispose(): number {
  return VULFRAM_CORE_DYLIB.vulfram_dispose();
}

function vulframInit(): number {
  return VULFRAM_CORE_DYLIB.vulfram_init();
}

function vulframReceiveQueue(): BufferResult {
  const ptrHolder = new BigUint64Array(1);
  const sizeHolder = new BigUint64Array(1);
  const result = VULFRAM_CORE_DYLIB.vulfram_receive_queue(
    ptr(ptrHolder),
    ptr(sizeHolder),
  );
  if (!sizeHolder[0]) {
    return { buffer: Buffer.alloc(0), result };
  }
  const srcPtr = Number(ptrHolder[0]) as Pointer;
  if (!srcPtr) {
    return { buffer: Buffer.alloc(0), result };
  }
  const buffer = Buffer.from(toArrayBuffer(srcPtr, 0, Number(sizeHolder[0])));

  return { buffer, result };
}

function vulframReceiveEvents(): BufferResult {
  const ptrHolder = new BigUint64Array(1);
  const sizeHolder = new BigUint64Array(1);
  const result = VULFRAM_CORE_DYLIB.vulfram_receive_events(
    ptr(ptrHolder),
    ptr(sizeHolder),
  );
  if (!sizeHolder[0]) {
    return { buffer: Buffer.alloc(0), result };
  }
  const srcPtr = Number(ptrHolder[0]) as Pointer;
  if (!srcPtr) {
    return { buffer: Buffer.alloc(0), result };
  }
  const buffer = Buffer.from(toArrayBuffer(srcPtr, 0, Number(sizeHolder[0])));

  return { buffer, result };
}

function vulframSendQueue(data: Uint8Array): number {
  const buffer = Buffer.isBuffer(data) ? data : Buffer.from(data);
  return VULFRAM_CORE_DYLIB.vulfram_send_queue(ptr(buffer), buffer.length);
}

function vulframTick(time: number, deltaTime: number): number {
  return VULFRAM_CORE_DYLIB.vulfram_tick(time, deltaTime);
}

function vulframUploadBuffer(
  id: number,
  uploadType: number,
  data: Uint8Array,
): number {
  const buffer = Buffer.isBuffer(data) ? data : Buffer.from(data);
  return VULFRAM_CORE_DYLIB.vulfram_upload_buffer(
    id,
    uploadType,
    ptr(buffer),
    buffer.length,
  );
}

function vulframGetProfiling(): BufferResult {
  const ptrHolder = new BigUint64Array(1);
  const sizeHolder = new BigUint64Array(1);
  const result = VULFRAM_CORE_DYLIB.vulfram_get_profiling(
    ptr(ptrHolder),
    ptr(sizeHolder),
  );
  if (!sizeHolder[0]) {
    return { buffer: Buffer.alloc(0), result };
  }
  const srcPtr = Number(ptrHolder[0]) as Pointer;
  if (!srcPtr) {
    return { buffer: Buffer.alloc(0), result };
  }
  const buffer = Buffer.from(toArrayBuffer(srcPtr, 0, Number(sizeHolder[0])));

  return { buffer, result };
}

export const VULFRAM_CORE = {
  vulframDispose,
  vulframInit,
  vulframReceiveQueue,
  vulframReceiveEvents,
  vulframSendQueue,
  vulframTick,
  vulframUploadBuffer,
  vulframGetProfiling,
};
