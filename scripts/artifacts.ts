import { createHash } from 'crypto';
import { mkdir, readFile, readdir, rm, writeFile } from 'fs/promises';
import { dirname, join } from 'path';
import {
  VULFRAM_R2_DEFAULT_BASE_URL,
  buildArtifactUrl,
  getArtifactFileName,
  parsePackageArtifactTarget,
  type VulframBinding,
  type VulframPlatform,
} from '../packages/transport-types/src/index';

type PackageJson = {
  name?: string;
  version?: string;
};

const rootDir = join(import.meta.dir, '..');
const ALL_NATIVE_PLATFORMS: Exclude<VulframPlatform, 'browser'>[] = [
  'linux-x64',
  'linux-arm64',
  'macos-x64',
  'macos-arm64',
  'windows-x64',
  'windows-arm64',
];

function env(name: string): string | undefined {
  return process.env?.[name];
}

function envBool(name: string): boolean {
  const raw = env(name)?.toLowerCase();
  return raw === '1' || raw === 'true' || raw === 'yes';
}

async function ensureDir(path: string): Promise<void> {
  await mkdir(path, { recursive: true });
}

async function ensureParent(path: string): Promise<void> {
  await ensureDir(dirname(path));
}

async function cleanDirectory(path: string): Promise<void> {
  await ensureDir(path);
  const entries = await readdir(path);
  await Promise.all(
    entries
      .filter((entry) => entry !== '.gitkeep')
      .map((entry) => rm(join(path, entry), { recursive: true, force: true })),
  );
}

async function sha256File(path: string): Promise<string> {
  const data = await readFile(path);
  return createHash('sha256').update(data).digest('hex');
}

async function readPackageVersion(packageDirName: string): Promise<string> {
  const packagePath = join(rootDir, 'packages', packageDirName, 'package.json');
  const raw = await readFile(packagePath, 'utf8');
  const pkg = JSON.parse(raw) as PackageJson;
  if (!pkg.version) {
    throw new Error(`Missing version in ${packagePath}`);
  }
  return pkg.version;
}

async function downloadFile(url: string, destination: string): Promise<void> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status} while downloading ${url}`);
  }

  const bytes = new Uint8Array(await response.arrayBuffer());
  await ensureParent(destination);
  await writeFile(destination, bytes);

  const hashUrl = `${url}.sha256`;
  const hashResponse = await fetch(hashUrl);
  if (!hashResponse.ok) return;

  const expected = (await hashResponse.text()).trim().split(/\s+/)[0] ?? '';
  if (!expected) return;

  const actual = await sha256File(destination);
  if (actual !== expected) {
    throw new Error(
      `SHA256 mismatch for ${url}: expected=${expected} actual=${actual}`,
    );
  }
}

async function ensureArtifact(config: {
  binding: VulframBinding;
  platform: VulframPlatform;
  artifact: string;
  destination: string;
  baseUrl: string;
  packageVersion: string;
}): Promise<void> {
  const { channel, artifactVersion } = parsePackageArtifactTarget(
    config.packageVersion,
  );

  const url = buildArtifactUrl({
    baseUrl: config.baseUrl,
    channel,
    artifactVersion,
    binding: config.binding,
    platform: config.platform,
    artifact: config.artifact,
  });

  await downloadFile(url, config.destination);
}

async function main(): Promise<void> {
  if (envBool('VULFRAM_TRANSPORT_SKIP_DOWNLOAD')) {
    console.log('[artifacts] Skipping downloads (VULFRAM_TRANSPORT_SKIP_DOWNLOAD=true).');
    return;
  }

  if (envBool('VULFRAM_TRANSPORT_OFFLINE')) {
    console.log('[artifacts] Offline mode enabled (VULFRAM_TRANSPORT_OFFLINE=true).');
    return;
  }

  const baseUrl = env('VULFRAM_TRANSPORT_R2_BASE_URL') ?? VULFRAM_R2_DEFAULT_BASE_URL;

  const bunVersion = await readPackageVersion('transport-bun');
  const napiVersion = await readPackageVersion('transport-napi');
  const browserVersion = await readPackageVersion('transport-browser');

  const browserArtifacts = [
    'vulfram_core.js',
    'vulfram_core.d.ts',
    'vulfram_core_bg.wasm',
    'vulfram_core_bg.wasm.d.ts',
  ] as const;

  await Promise.all([
    cleanDirectory(join(rootDir, 'packages', 'transport-bun', 'lib')),
    cleanDirectory(join(rootDir, 'packages', 'transport-napi', 'lib')),
    cleanDirectory(join(rootDir, 'packages', 'transport-browser', 'lib')),
  ]);

  const tasks: Array<Promise<void>> = [];

  for (const platform of ALL_NATIVE_PLATFORMS) {
    const ffiName = getArtifactFileName('ffi', platform);
    const napiName = getArtifactFileName('napi', platform);

    tasks.push(
      ensureArtifact({
        binding: 'ffi',
        platform,
        artifact: ffiName,
        destination: join(
          rootDir,
          'packages',
          'transport-bun',
          'lib',
          platform,
          ffiName,
        ),
        baseUrl,
        packageVersion: bunVersion,
      }),
    );

    tasks.push(
      ensureArtifact({
        binding: 'napi',
        platform,
        artifact: napiName,
        destination: join(
          rootDir,
          'packages',
          'transport-napi',
          'lib',
          platform,
          napiName,
        ),
        baseUrl,
        packageVersion: napiVersion,
      }),
    );
  }

  for (const artifact of browserArtifacts) {
    tasks.push(
      ensureArtifact({
        binding: 'wasm',
        platform: 'browser',
        artifact,
        destination: join(
          rootDir,
          'packages',
          'transport-browser',
          'lib',
          artifact,
        ),
        baseUrl,
        packageVersion: browserVersion,
      }),
    );
  }

  const results = await Promise.allSettled(tasks);
  const failures = results.filter((result) => result.status === 'rejected');

  if (failures.length > 0) {
    console.warn(`[artifacts] ${failures.length} artifact download(s) failed.`);
    for (const failure of failures) {
      console.warn(`- ${(failure as PromiseRejectedResult).reason}`);
    }
    return;
  }

  console.log('[artifacts] Transport artifacts downloaded successfully.');
}

main().catch((error) => {
  console.warn('[artifacts] Unexpected failure:', error);
  process.exitCode = 1;
});
