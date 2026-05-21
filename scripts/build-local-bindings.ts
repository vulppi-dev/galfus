import { copyFile, mkdir } from 'fs/promises';
import { dirname, join } from 'path';
import { Command } from 'commander';
import {
  getArtifactFileName,
  resolveNativePlatform,
  type GalfusPlatform
} from '../packages/transport-types/src/index';

type BuildMode = 'debug' | 'release';

type CliOptions = {
  mode: BuildMode;
  skipBun: boolean;
  skipNapi: boolean;
  skipWasm: boolean;
};

const rootDir = join(import.meta.dir, '..');
const cargoTargetDir = join(rootDir, 'build', 'cargo-target');

function logStep(message: string): void {
  console.log(`[build-local-bindings] ${message}`);
}

async function ensureDir(path: string): Promise<void> {
  await mkdir(path, { recursive: true });
}

function getNativePlatform(): Exclude<GalfusPlatform, 'browser'> {
  return resolveNativePlatform();
}

function getNativeLibraryExtension(platform: Exclude<GalfusPlatform, 'browser'>): string {
  if (platform.startsWith('windows')) return 'dll';
  if (platform.startsWith('macos')) return 'dylib';
  return 'so';
}

function getCargoProfileDir(mode: BuildMode): string {
  return mode === 'release' ? 'release' : 'debug';
}

async function runCommand(label: string, cmd: string[], cwd = rootDir): Promise<void> {
  logStep(label);
  const proc = Bun.spawn({
    cmd,
    cwd,
    stdout: 'inherit',
    stderr: 'inherit',
    stdin: 'inherit',
    env: {
      ...process.env,
      CARGO_TARGET_DIR: cargoTargetDir
    }
  });
  const exitCode = await proc.exited;
  if (exitCode !== 0) {
    throw new Error(`Command failed (${exitCode}): ${cmd.join(' ')}`);
  }
}

async function commandExists(name: string): Promise<boolean> {
  const proc = Bun.spawn({
    cmd: ['bash', '-lc', `command -v ${name}`],
    cwd: rootDir,
    stdout: 'ignore',
    stderr: 'ignore'
  });
  return (await proc.exited) === 0;
}

async function ensureWasmToolchain(): Promise<void> {
  await runCommand('Ensuring wasm32 target', ['rustup', 'target', 'add', 'wasm32-unknown-unknown']);

  if (await commandExists('wasm-bindgen')) {
    return;
  }

  await runCommand('Installing wasm-bindgen-cli 0.2.114', [
    'cargo',
    'install',
    'wasm-bindgen-cli',
    '--version',
    '0.2.114'
  ]);
}

async function copyArtifact(source: string, destination: string): Promise<void> {
  await ensureDir(dirname(destination));
  await copyFile(source, destination);
  logStep(`Wrote ${destination.replace(`${rootDir}/`, '')}`);
}

async function buildBunFfi(
  platform: Exclude<GalfusPlatform, 'browser'>,
  mode: BuildMode
): Promise<void> {
  const profileDir = getCargoProfileDir(mode);
  const extension = getNativeLibraryExtension(platform);
  const source = join(cargoTargetDir, profileDir, `libgalfus_bindings_ffi.${extension}`);
  const destination = join(
    rootDir,
    'packages',
    'transport-bun',
    'dist',
    platform,
    getArtifactFileName('ffi', platform)
  );

  const cargoArgs = ['cargo', 'build', '-p', 'galfus-bindings-ffi'];
  if (mode === 'release') {
    cargoArgs.push('--release');
  }

  await runCommand(`Building Bun FFI (${mode})`, cargoArgs);
  await copyArtifact(source, destination);
}

async function buildNapi(
  platform: Exclude<GalfusPlatform, 'browser'>,
  mode: BuildMode
): Promise<void> {
  const profileDir = getCargoProfileDir(mode);
  const extension = getNativeLibraryExtension(platform);
  const libPrefix = platform.startsWith('windows') ? '' : 'lib';
  const source = join(cargoTargetDir, profileDir, `${libPrefix}galfus_bindings_napi.${extension}`);
  const destination = join(
    rootDir,
    'packages',
    'transport-napi',
    'dist',
    platform,
    getArtifactFileName('napi', platform)
  );

  const cargoArgs = ['cargo', 'build', '-p', 'galfus-bindings-napi'];
  if (mode === 'release') {
    cargoArgs.push('--release');
  }

  await runCommand(`Building N-API (${mode})`, cargoArgs);
  await copyArtifact(source, destination);
}

async function buildWasm(mode: BuildMode): Promise<void> {
  const profileDir = getCargoProfileDir(mode);
  const crateOutput = join(
    cargoTargetDir,
    'wasm32-unknown-unknown',
    profileDir,
    'galfus_bindings_wasm.wasm'
  );
  const destinationDir = join(rootDir, 'packages', 'transport-browser', 'dist');

  const cargoArgs = [
    'cargo',
    'build',
    '-p',
    'galfus-bindings-wasm',
    '--target',
    'wasm32-unknown-unknown'
  ];
  if (mode === 'release') {
    cargoArgs.push('--release');
  }

  await ensureWasmToolchain();
  await runCommand(`Building WASM (${mode})`, cargoArgs);
  await ensureDir(destinationDir);
  await runCommand('Generating wasm-bindgen browser bindings', [
    'wasm-bindgen',
    '--target',
    'web',
    '--out-dir',
    destinationDir,
    '--out-name',
    'galfus_core',
    crateOutput
  ]);
}

async function parseOptions(): Promise<CliOptions> {
  const program = new Command();
  program
    .name('build-local-bindings')
    .description('Build local core artifacts for Bun FFI, N-API, and browser WASM.')
    .option('--mode <mode>', 'Build mode: debug or release.', 'release')
    .option('--skip-bun', 'Skip Bun FFI build.', false)
    .option('--skip-napi', 'Skip N-API build.', false)
    .option('--skip-wasm', 'Skip browser WASM build.', false);

  await program.parseAsync(process.argv);

  const options = program.opts<CliOptions>();
  if (options.mode !== 'debug' && options.mode !== 'release') {
    throw new Error(`Invalid mode "${options.mode}". Expected "debug" or "release".`);
  }

  return options;
}

async function main(): Promise<void> {
  const options = await parseOptions();
  const platform = getNativePlatform();

  logStep(`Detected local platform: ${platform}`);
  logStep(`Build mode: ${options.mode}`);

  if (!options.skipBun) {
    await buildBunFfi(platform, options.mode);
  }

  if (!options.skipNapi) {
    await buildNapi(platform, options.mode);
  }

  if (!options.skipWasm) {
    await buildWasm(options.mode);
  }

  const builtTargets: string[] = [];
  if (!options.skipBun) builtTargets.push(`bun:${platform}`);
  if (!options.skipNapi) builtTargets.push(`napi:${platform}`);
  if (!options.skipWasm) builtTargets.push('wasm:browser');

  logStep(`Done. Built ${builtTargets.join(', ') || 'nothing'}.`);
}

main().catch((error) => {
  console.error('[build-local-bindings] Failed:', error);
  process.exitCode = 1;
});
