import {
  detectRuntime,
  getArtifactFileName,
  resolveNativePlatform,
  selectPlatformLoader,
  type PlatformLoaderMap,
} from '@vulfram/transport-types';
import type { BufferResult } from './types';
import { createRequire } from 'module';
import { dirname, join } from 'path';
import { existsSync } from 'fs';
import { fileURLToPath } from 'url';

const requireNative = createRequire(import.meta.url);

const bundledLoaders: PlatformLoaderMap<{ default: string }> = {
  darwin: {
    arm64: () =>
      // @ts-expect-error
      import('../../dist/macos-arm64/vulfram_core.node', {
        with: { type: 'file' }
      }),
    x64: () =>
      // @ts-expect-error
      import('../../dist/macos-x64/vulfram_core.node', {
        with: { type: 'file' }
      })
  },
  linux: {
    arm64: () =>
      // @ts-expect-error
      import('../../dist/linux-arm64/vulfram_core.node', {
        with: { type: 'file' }
      }),
    x64: () =>
      // @ts-expect-error
      import('../../dist/linux-x64/vulfram_core.node', {
        with: { type: 'file' }
      })
  },
  win32: {
    arm64: () =>
      // @ts-expect-error
      import('../../dist/windows-arm64/vulfram_core.node', {
        with: { type: 'file' }
      }),
    x64: () =>
      // @ts-expect-error
      import('../../dist/windows-x64/vulfram_core.node', {
        with: { type: 'file' }
      })
  }
};

function getExpectedLocalArtifact(): string {
  try {
    const platform = resolveNativePlatform();
    const filename = getArtifactFileName('napi', platform);
    return `../../dist/${platform}/${filename}`;
  } catch {
    return '../../dist/<platform>/vulfram_core.node';
  }
}

async function resolveNativeModulePath(): Promise<string> {
  const platform = resolveNativePlatform();
  const filename = getArtifactFileName('napi', platform);
  const moduleDir = dirname(fileURLToPath(import.meta.url));
  const candidates = [
    join(moduleDir, '..', '..', 'dist', platform, filename),
    join(moduleDir, platform, filename),
    join(moduleDir, '..', platform, filename),
  ];

  try {
    const resolved = candidates.find((candidate) => existsSync(candidate));
    if (resolved) {
      return resolved;
    }

    const importLoader = selectPlatformLoader(bundledLoaders, 'N-API');
    return (await importLoader()).default;
  } catch (error) {
    const runtime = detectRuntime();
    const expectedArtifact = getExpectedLocalArtifact();
    throw new Error(
      `Failed to load bundled N-API artifact (runtime=${runtime.runtime}, platform=${runtime.platform ?? 'unknown'}, arch=${runtime.arch ?? 'unknown'}, expected=${expectedArtifact}): ${String(error)}`
    );
  }
}

const modulePath = await resolveNativeModulePath();
const raw = requireNative(modulePath) as {
  vulframInit: () => number;
  vulframDispose: () => number;
  vulframSendQueue: (buffer: Buffer) => number;
  vulframReceiveQueue: () => BufferResult;
  vulframReceiveEvents: () => BufferResult;
  vulframUploadBuffer: (id: number, uploadType: number, buffer: Buffer) => number;
  vulframTick: (timeMs: number, deltaMs: number) => number;
  vulframGetProfiling: () => BufferResult;
};

export const VULFRAM_CORE = {
  vulframInit: () => raw.vulframInit(),
  vulframDispose: () => raw.vulframDispose(),
  vulframReceiveQueue: () => raw.vulframReceiveQueue(),
  vulframReceiveEvents: () => raw.vulframReceiveEvents(),
  vulframTick: (timeMs: number, deltaMs: number) => raw.vulframTick(timeMs, deltaMs),
  vulframGetProfiling: () => raw.vulframGetProfiling(),
  vulframSendQueue: (buffer: Uint8Array) => {
    const data = Buffer.isBuffer(buffer) ? buffer : Buffer.from(buffer);
    return raw.vulframSendQueue(data);
  },
  vulframUploadBuffer: (id: number, uploadType: number, buffer: Uint8Array) => {
    const data = Buffer.isBuffer(buffer) ? buffer : Buffer.from(buffer);
    return raw.vulframUploadBuffer(id, uploadType, data);
  }
};
