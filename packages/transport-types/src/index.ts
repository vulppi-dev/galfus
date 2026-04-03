export type TransportBuffer = Uint8Array;

export type BufferResult = {
  buffer: TransportBuffer;
  result: number;
};

export type EngineTransport = {
  vulframInit: () => number;
  vulframDispose: () => number;
  vulframSendQueue: (buffer: TransportBuffer) => number;
  vulframReceiveQueue: () => BufferResult;
  vulframReceiveEvents: () => BufferResult;
  vulframUploadBuffer: (
    id: number,
    uploadType: number,
    buffer: TransportBuffer,
  ) => number;
  vulframTick: (timeMs: number, deltaMs: number) => number;
  vulframGetProfiling: () => BufferResult;
};

export type EngineTransportFactory = () => EngineTransport;

export type VulframChannel = 'alpha' | 'beta' | 'release';
export type VulframBinding = 'ffi' | 'napi' | 'wasm';
export type VulframPlatform =
  | 'linux-x64'
  | 'linux-arm64'
  | 'macos-x64'
  | 'macos-arm64'
  | 'windows-x64'
  | 'windows-arm64'
  | 'browser';

export type RuntimeKind = 'node' | 'bun' | 'deno' | 'unknown';

export type RuntimeInfo = {
  runtime: RuntimeKind;
  version: string | null;
  platform: string | null;
  arch: string | null;
};

export const VULFRAM_R2_DEFAULT_BASE_URL =
  'https://pub-95922dbd81b344a893425215a2695b88.r2.dev';
export const VULFRAM_ARTIFACT_PREFIX = 'v1';
export const VULFRAM_DEFAULT_CHANNEL: VulframChannel = 'alpha';
export const VULFRAM_DEFAULT_BINDINGS: readonly VulframBinding[] = [
  'ffi',
  'napi',
  'wasm',
];

export type PlatformLoaderMap<T> = Record<
  string,
  Record<string, () => Promise<T>>
>;

export function detectRuntime(): RuntimeInfo {
  if (
    // @ts-ignore
    typeof globalThis.Deno !== 'undefined' &&
    // @ts-ignore
    typeof globalThis.Deno?.version?.deno === 'string'
  ) {
    // @ts-ignore
    const deno = globalThis.Deno;
    return {
      runtime: 'deno',
      version: deno.version.deno,
      platform: deno.build?.os ?? null,
      arch: deno.build?.arch ?? null,
    };
  }

  if (
    // @ts-ignore
    typeof globalThis.Bun !== 'undefined' &&
    // @ts-ignore
    typeof globalThis.Bun?.version === 'string'
  ) {
    return {
      runtime: 'bun',
      // @ts-ignore
      version: globalThis.Bun.version,
      // @ts-ignore
      platform: typeof process !== 'undefined' ? process.platform : null,
      // @ts-ignore
      arch: typeof process !== 'undefined' ? process.arch : null,
    };
  }

  if (
    // @ts-ignore
    typeof globalThis.process !== 'undefined' &&
    // @ts-ignore
    typeof globalThis.process?.versions?.node === 'string'
  ) {
    // @ts-ignore
    const proc = globalThis.process;
    return {
      runtime: 'node',
      version: proc.versions.node,
      platform: proc.platform ?? null,
      arch: proc.arch ?? null,
    };
  }

  return {
    runtime: 'unknown',
    version: null,
    platform: null,
    arch: null,
  };
}

export function selectPlatformLoader<T>(
  loaders: PlatformLoaderMap<T>,
  artifactKind: string,
): () => Promise<T> {
  const runtime = detectRuntime();
  const platformKey = runtime.platform ?? '';
  const archKey = runtime.arch ?? '';
  const byPlatform = loaders[platformKey];
  const selected = byPlatform?.[archKey];

  if (selected) return selected;

  throw new Error(
    `${artifactKind} build not found for the current runtime: ${JSON.stringify(
      runtime,
    )}`,
  );
}

export function resolveNativePlatform(runtime = detectRuntime()): Exclude<
  VulframPlatform,
  'browser'
> {
  const platform = runtime.platform;
  const arch = runtime.arch;

  if (platform === 'linux' && arch === 'x64') return 'linux-x64';
  if (platform === 'linux' && arch === 'arm64') return 'linux-arm64';
  if (platform === 'darwin' && arch === 'x64') return 'macos-x64';
  if (platform === 'darwin' && arch === 'arm64') return 'macos-arm64';
  if (platform === 'win32' && arch === 'x64') return 'windows-x64';
  if (platform === 'win32' && arch === 'arm64') return 'windows-arm64';

  throw new Error(
    `Unsupported native platform for Vulfram transports: ${JSON.stringify(
      runtime,
    )}`,
  );
}

export function getArtifactFileName(
  binding: VulframBinding,
  platform: VulframPlatform,
): string {
  if (binding === 'napi') return 'vulfram_core.node';
  if (binding === 'wasm') return 'vulfram_core_bg.wasm';
  if (binding === 'ffi' && platform.startsWith('windows'))
    return 'vulfram_core.dll';
  if (binding === 'ffi' && platform.startsWith('macos'))
    return 'vulfram_core.dylib';
  if (binding === 'ffi') return 'vulfram_core.so';
  if (platform.startsWith('windows')) return 'vulfram_core.dll';
  if (platform.startsWith('macos')) return 'vulfram_core.dylib';
  return 'vulfram_core.so';
}

export function buildArtifactPath(config: {
  channel?: VulframChannel;
  artifactVersion: string;
  binding: VulframBinding;
  platform: VulframPlatform;
  artifact?: string;
  prefix?: string;
}): string {
  const channel = config.channel ?? VULFRAM_DEFAULT_CHANNEL;
  const prefix = config.prefix ?? VULFRAM_ARTIFACT_PREFIX;
  const artifact =
    config.artifact ?? getArtifactFileName(config.binding, config.platform);
  return [
    prefix,
    channel,
    config.artifactVersion,
    config.binding,
    config.platform,
    artifact,
  ].join('/');
}

export function buildArtifactUrl(config: {
  baseUrl?: string;
  channel?: VulframChannel;
  artifactVersion: string;
  binding: VulframBinding;
  platform: VulframPlatform;
  artifact?: string;
  prefix?: string;
}): string {
  const base = (config.baseUrl ?? VULFRAM_R2_DEFAULT_BASE_URL).replace(
    /\/+$/,
    '',
  );
  return `${base}/${buildArtifactPath(config)}`;
}

export function parsePackageArtifactTarget(packageVersion: string): {
  channel: VulframChannel;
  artifactVersion: string;
} {
  const normalized = packageVersion.trim();
  const match = normalized.match(
    /^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$/,
  );

  if (!match) {
    throw new Error(
      `Invalid package version "${packageVersion}". Expected semver format "X.Y.Z[-tag]".`,
    );
  }

  const major = match[1]!;
  const minor = match[2]!;
  const pre = (match[4] ?? '').toLowerCase();

  let channel: VulframChannel = 'release';
  if (pre.includes('alpha')) channel = 'alpha';
  else if (pre.includes('beta')) channel = 'beta';

  return {
    channel,
    artifactVersion: `v${major}.${minor}`,
  };
}
