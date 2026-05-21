export type TransportBuffer = Uint8Array;

export type BufferResult = {
  buffer: TransportBuffer;
  result: number;
};

export type EngineTransport = {
  galfusInit: () => number;
  galfusDispose: () => number;
  galfusSendQueue: (buffer: TransportBuffer) => number;
  galfusReceiveQueue: () => BufferResult;
  galfusReceiveEvents: () => BufferResult;
  galfusUploadBuffer: (id: number, uploadType: number, buffer: TransportBuffer) => number;
  galfusTick: (timeMs: number, deltaMs: number) => number;
  galfusGetProfiling: () => BufferResult;
};

export type EngineTransportFactory = () => EngineTransport;

export type GalfusChannel = 'alpha' | 'beta' | 'next' | 'latest';
export type GalfusBinding = 'ffi' | 'napi' | 'wasm';
export type GalfusPlatform =
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

export const GALFUS_R2_DEFAULT_BASE_URL = 'https://pub-95922dbd81b344a893425215a2695b88.r2.dev';
export const GALFUS_ARTIFACT_PREFIX = 'v1';
export const GALFUS_DEFAULT_CHANNEL: GalfusChannel = 'alpha';
export const GALFUS_DEFAULT_BINDINGS: readonly GalfusBinding[] = ['ffi', 'napi', 'wasm'];

export type PlatformLoaderMap<T> = Record<string, Record<string, () => Promise<T>>>;

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
      arch: deno.build?.arch ?? null
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
      arch: typeof process !== 'undefined' ? process.arch : null
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
      arch: proc.arch ?? null
    };
  }

  return {
    runtime: 'unknown',
    version: null,
    platform: null,
    arch: null
  };
}

export function selectPlatformLoader<T>(
  loaders: PlatformLoaderMap<T>,
  artifactKind: string
): () => Promise<T> {
  const runtime = detectRuntime();
  const platformKey = runtime.platform ?? '';
  const archKey = runtime.arch ?? '';
  const byPlatform = loaders[platformKey];
  const selected = byPlatform?.[archKey];

  if (selected) return selected;

  throw new Error(
    `${artifactKind} build not found for the current runtime: ${JSON.stringify(runtime)}`
  );
}

export function resolveNativePlatform(
  runtime: RuntimeInfo = detectRuntime()
): Exclude<GalfusPlatform, 'browser'> {
  const platform = runtime.platform;
  const arch = runtime.arch;

  if (platform === 'linux' && arch === 'x64') return 'linux-x64';
  if (platform === 'linux' && arch === 'arm64') return 'linux-arm64';
  if (platform === 'darwin' && arch === 'x64') return 'macos-x64';
  if (platform === 'darwin' && arch === 'arm64') return 'macos-arm64';
  if (platform === 'win32' && arch === 'x64') return 'windows-x64';
  if (platform === 'win32' && arch === 'arm64') return 'windows-arm64';

  throw new Error(`Unsupported native platform for Galfus transports: ${JSON.stringify(runtime)}`);
}

export function getArtifactFileName(binding: GalfusBinding, platform: GalfusPlatform): string {
  if (binding === 'napi') return 'galfus_core.node';
  if (binding === 'wasm') return 'galfus_core_bg.wasm';
  if (binding === 'ffi' && platform.startsWith('windows')) return 'galfus_core.dll';
  if (binding === 'ffi' && platform.startsWith('macos')) return 'galfus_core.dylib';
  if (binding === 'ffi') return 'galfus_core.so';
  if (platform.startsWith('windows')) return 'galfus_core.dll';
  if (platform.startsWith('macos')) return 'galfus_core.dylib';
  return 'galfus_core.so';
}

export function buildArtifactPath(config: {
  channel?: GalfusChannel;
  artifactVersion: string;
  binding: GalfusBinding;
  platform: GalfusPlatform;
  artifact?: string;
  prefix?: string;
}): string {
  const channel = config.channel ?? GALFUS_DEFAULT_CHANNEL;
  const prefix = config.prefix ?? GALFUS_ARTIFACT_PREFIX;
  const artifact = config.artifact ?? getArtifactFileName(config.binding, config.platform);
  return [prefix, channel, config.artifactVersion, config.binding, config.platform, artifact].join(
    '/'
  );
}

export function buildArtifactUrl(config: {
  baseUrl?: string;
  channel?: GalfusChannel;
  artifactVersion: string;
  binding: GalfusBinding;
  platform: GalfusPlatform;
  artifact?: string;
  prefix?: string;
}): string {
  const base = (config.baseUrl ?? GALFUS_R2_DEFAULT_BASE_URL).replace(/\/+$/, '');
  return `${base}/${buildArtifactPath(config)}`;
}

export function parsePackageArtifactTarget(packageVersion: string): {
  channel: GalfusChannel;
  artifactVersion: string;
} {
  const normalized = packageVersion.trim();
  const match = normalized.match(
    /^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$/
  );

  if (!match) {
    throw new Error(
      `Invalid package version "${packageVersion}". Expected semver format "X.Y.Z[-tag]".`
    );
  }

  const major = match[1]!;
  const minor = match[2]!;
  const patch = match[3]!;
  const pre = (match[4] ?? '').toLowerCase();

  let channel: GalfusChannel = 'latest';
  if (pre.includes('alpha')) channel = 'alpha';
  else if (pre.includes('beta')) channel = 'beta';
  else if (pre.includes('next')) channel = 'next';

  return {
    channel,
    artifactVersion: `v${major}.${minor}.${patch}`
  };
}
