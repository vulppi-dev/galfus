import { join } from 'path';

const JSR_PACKAGE_DIRS = [
  'packages/transport-types',
  'packages/transport-browser',
  'packages/engine',
  'packages/gltf-loader',
  'packages/camera-control'
] as const;

async function runJsrDryRun(cwd: string): Promise<void> {
  console.log(`[lint] jsr publish --dry-run (${cwd})`);

  const proc = Bun.spawn({
    cmd: ['bunx', 'jsr', 'publish', '--dry-run', '--allow-dirty'],
    cwd,
    stdout: 'inherit',
    stderr: 'inherit',
    stdin: 'inherit'
  });

  const exitCode = await proc.exited;
  if (exitCode !== 0) {
    throw new Error(`JSR dry-run failed in ${cwd} with exit code ${exitCode}`);
  }
}

async function main(): Promise<void> {
  const rootDir = join(import.meta.dir, '..');

  for (const packageDir of JSR_PACKAGE_DIRS) {
    await runJsrDryRun(join(rootDir, packageDir));
  }
}

main().catch((error) => {
  console.error('[lint] Failed:', error);
  process.exitCode = 1;
});
