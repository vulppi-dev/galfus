import { build } from 'bun';
import { mkdirSync, rmSync } from 'fs';
import { join } from 'path';

const rootDir = process.cwd();
const outDir = join(rootDir, 'dist');

rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });

const result = await build({
  entrypoints: [join(rootDir, 'src', 'index.ts')],
  outdir: outDir,
  target: 'node',
  format: 'esm',
  minify: false,
});

if (!result.success) {
  const firstError = result.logs.find((log) => log.level === 'error');
  const reason = firstError?.message ?? 'build failed';
  throw new Error(reason);
}

console.log('Bundled transport-napi to dist/.');
