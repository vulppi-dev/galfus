import {
  cpSync,
  existsSync,
  mkdirSync,
  renameSync,
  rmSync,
  statSync,
} from 'fs';
import { glob } from 'glob';
import { basename, join } from 'path';
import minimist from 'minimist';
import { build } from 'bun';

type TargetSpec = {
  key: string;
  bunTarget: string;
  libDir: string;
  libExt: string;
  exeName: string;
};

const TARGETS: TargetSpec[] = [
  {
    key: 'darwin-arm64',
    bunTarget: 'bun-darwin-arm64',
    libDir: 'macos-arm64',
    libExt: 'dylib',
    exeName: 'vulfram-demo',
  },
  {
    key: 'darwin-x64',
    bunTarget: 'bun-darwin-x64',
    libDir: 'macos-x64',
    libExt: 'dylib',
    exeName: 'vulfram-demo',
  },
  {
    key: 'linux-arm64',
    bunTarget: 'bun-linux-arm64',
    libDir: 'linux-arm64',
    libExt: 'so',
    exeName: 'vulfram-demo',
  },
  {
    key: 'linux-x64',
    bunTarget: 'bun-linux-x64',
    libDir: 'linux-x64',
    libExt: 'so',
    exeName: 'vulfram-demo',
  },
  // { /* NOT SUPPORTED */
  //   key: 'win32-arm64',
  //   bunTarget: 'bun-windows-arm64',
  //   libDir: 'windows-arm64',
  //   libExt: 'dll',
  //   exeName: 'vulfram-demo.exe',
  // },
  {
    key: 'win32-x64',
    bunTarget: 'bun-windows-x64',
    libDir: 'windows-x64',
    libExt: 'dll',
    exeName: 'vulfram-demo.exe',
  },
];

function parseTargets(args: string[]): TargetSpec[] {
  const parsed = minimist(args);
  const targetsArg = (parsed.target ?? parsed.targets ?? '') as
    | string
    | string[];
  const requested = Array.isArray(targetsArg)
    ? targetsArg.flatMap((value) => value.split(',').map((v) => v.trim()))
    : targetsArg
        .split(',')
        .map((v) => v.trim())
        .filter(Boolean);

  if (requested.length === 0) return TARGETS;
  const filtered = TARGETS.filter((t) => requested.includes(t.key));
  if (filtered.length === 0) {
    throw new Error(
      `No targets matched: ${requested.join(', ')}. Available: ${TARGETS.map(
        (t) => t.key,
      ).join(', ')}`,
    );
  }
  return filtered;
}

const rootDir = process.cwd();
const outRoot = join(rootDir, 'dist');
const targets = parseTargets(process.argv.slice(2));
const failedTargets: string[] = [];
const builtTargets: string[] = [];

for (const target of targets) {
  const outDir = join(outRoot, target.key);
  rmSync(outDir, { recursive: true, force: true });
  mkdirSync(outDir, { recursive: true });

  const outFile = join(outDir, target.exeName);
  const libs = await glob(`**/${target.libDir}/**/*.${target.libExt}`);

  const compileResult = await build({
    entrypoints: [join(rootDir, 'src', 'index.ts'), ...libs],
    outdir: outDir,
    target: target.bunTarget as any,
    compile: true,
    minify: false,
  });
  if (!compileResult.success) {
    const firstError = compileResult.logs.find((log) => log.level === 'error');
    const reason = firstError?.message ?? 'build failed';
    console.warn(`Skipping target ${target.key}: ${reason}`);
    failedTargets.push(target.key);
    continue;
  }
  const outputPath = compileResult.outputs[0]?.path;
  if (outputPath && basename(outputPath) !== target.exeName) {
    renameSync(outputPath, outFile);
  }

  builtTargets.push(target.key);
}

console.log(`Build complete: ${builtTargets.join(', ')}`);
if (failedTargets.length > 0) {
  console.warn(`Build skipped for: ${failedTargets.join(', ')}`);
}
