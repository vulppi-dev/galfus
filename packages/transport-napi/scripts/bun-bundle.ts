import { build } from 'bun';
import { mkdirSync, readdirSync, rmSync, writeFileSync, existsSync } from 'fs';
import { join } from 'path';

const rootDir = process.cwd();
const outDir = join(rootDir, 'dist');
const PLATFORM_DIRS = [
  'linux-arm64',
  'linux-x64',
  'macos-arm64',
  'macos-x64',
  'windows-arm64',
  'windows-x64'
] as const;

mkdirSync(outDir, { recursive: true });

for (const entry of readdirSync(outDir, { withFileTypes: true })) {
  if (entry.isDirectory()) continue;
  if (!entry.name.startsWith('index.') && !entry.name.startsWith('vulfram_core-')) continue;
  rmSync(join(outDir, entry.name), { recursive: true, force: true });
}

for (const platform of PLATFORM_DIRS) {
  const platformDir = join(outDir, platform);
  const artifactPath = join(platformDir, 'vulfram_core.node');
  mkdirSync(platformDir, { recursive: true });
  if (!existsSync(artifactPath)) {
    writeFileSync(artifactPath, new Uint8Array());
  }
}

const result = await build({
  entrypoints: [join(rootDir, 'src', 'index.ts')],
  outdir: outDir,
  target: 'node',
  format: 'esm',
  minify: false
});

if (!result.success) {
  const firstError = result.logs.find((log) => log.level === 'error');
  const reason = firstError?.message ?? 'build failed';
  throw new Error(reason);
}

writeFileSync(
  join(outDir, 'index.d.ts'),
  "import type { EngineTransportFactory } from '@vulfram/transport-types';\nexport declare const transportNapi: EngineTransportFactory;\n"
);

console.log('Bundled transport-napi to dist/.');
