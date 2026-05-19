import { dlopen, ptr, toArrayBuffer, type Pointer } from 'bun:ffi';
import {
  detectRuntime,
  getArtifactFileName,
  resolveNativePlatform,
  selectPlatformLoader,
  type PlatformLoaderMap
} from '@galfus/transport-types';
import type { BufferResult } from './types';

const loaders: PlatformLoaderMap<{ default: string }> = {
  darwin: {
    arm64: () =>
      // @ts-expect-error
      import('../../dist/macos-arm64/galfus_core.dylib', {
        with: { type: 'file' }
      }),
    x64: () =>
      // @ts-expect-error
      import('../../dist/macos-x64/galfus_core.dylib', {
        with: { type: 'file' }
      })
  },
  linux: {
    arm64: () =>
      // @ts-expect-error
      import('../../dist/linux-arm64/galfus_core.so', {
        with: { type: 'file' }
      }),
    x64: () =>
      // @ts-expect-error
      import('../../dist/linux-x64/galfus_core.so', {
        with: { type: 'file' }
      })
  },
  win32: {
    arm64: () =>
      // @ts-expect-error
      import('../../dist/windows-arm64/galfus_core.dll', {
        with: { type: 'file' }
      }),
    x64: () =>
      // @ts-expect-error
      import('../../dist/windows-x64/galfus_core.dll', {
        with: { type: 'file' }
      })
  }
};

function getExpectedLocalArtifact(): string {
  try {
    const platform = resolveNativePlatform();
    const filename = getArtifactFileName('ffi', platform);
    return `../../dist/${platform}/${filename}`;
  } catch {
    return '../../dist/<platform>/galfus_core.<dll|dylib|so>';
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
      `Failed to load bundled FFI artifact (runtime=${runtime.runtime}, platform=${runtime.platform ?? 'unknown'}, arch=${runtime.arch ?? 'unknown'}, expected=${expectedArtifact}): ${String(error)}`
    );
  }
}

const lib = await resolveNativeLibraryPath();

const { symbols: GALFUS_CORE_DYLIB, close } = dlopen(lib, {
  galfus_init: { args: [], returns: 'u32' },
  galfus_dispose: { args: [], returns: 'u32' },
  galfus_send_queue: { args: ['ptr', 'usize'], returns: 'u32' },
  galfus_receive_queue: { args: ['ptr', 'ptr'], returns: 'u32' },
  galfus_receive_events: { args: ['ptr', 'ptr'], returns: 'u32' },
  galfus_upload_buffer: {
    args: ['u64', 'u32', 'ptr', 'usize'],
    returns: 'u32'
  },
  galfus_tick: { args: ['u64', 'u32'], returns: 'u32' },
  galfus_get_profiling: { args: ['ptr', 'ptr'], returns: 'u32' }
});

process.once('beforeExit', () => {
  close();
});

function galfusDispose(): number {
  return GALFUS_CORE_DYLIB.galfus_dispose();
}

function galfusInit(): number {
  return GALFUS_CORE_DYLIB.galfus_init();
}

function galfusReceiveQueue(): BufferResult {
  const ptrHolder = new BigUint64Array(1);
  const sizeHolder = new BigUint64Array(1);
  const result = GALFUS_CORE_DYLIB.galfus_receive_queue(ptr(ptrHolder), ptr(sizeHolder));
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

function galfusReceiveEvents(): BufferResult {
  const ptrHolder = new BigUint64Array(1);
  const sizeHolder = new BigUint64Array(1);
  const result = GALFUS_CORE_DYLIB.galfus_receive_events(ptr(ptrHolder), ptr(sizeHolder));
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

function galfusSendQueue(data: Uint8Array): number {
  const buffer = Buffer.isBuffer(data) ? data : Buffer.from(data);
  return GALFUS_CORE_DYLIB.galfus_send_queue(ptr(buffer), buffer.length);
}

function galfusTick(time: number, deltaTime: number): number {
  return GALFUS_CORE_DYLIB.galfus_tick(time, deltaTime);
}

function galfusUploadBuffer(id: number, uploadType: number, data: Uint8Array): number {
  const buffer = Buffer.isBuffer(data) ? data : Buffer.from(data);
  return GALFUS_CORE_DYLIB.galfus_upload_buffer(id, uploadType, ptr(buffer), buffer.length);
}

function galfusGetProfiling(): BufferResult {
  const ptrHolder = new BigUint64Array(1);
  const sizeHolder = new BigUint64Array(1);
  const result = GALFUS_CORE_DYLIB.galfus_get_profiling(ptr(ptrHolder), ptr(sizeHolder));
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

export const GALFUS_CORE = {
  galfusDispose,
  galfusInit,
  galfusReceiveQueue,
  galfusReceiveEvents,
  galfusSendQueue,
  galfusTick,
  galfusUploadBuffer,
  galfusGetProfiling
};
