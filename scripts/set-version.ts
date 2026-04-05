import { readFile, writeFile } from 'fs/promises';
import { join } from 'path';
import { Command } from 'commander';

type PackageJson = {
  name?: string;
  version?: string;
  [key: string]: unknown;
};

const TARGET_PACKAGES = ['transport-bun', 'transport-napi', 'transport-browser'] as const;

async function parseVersionArg(argv: string[]): Promise<string> {
  const program = new Command();
  program
    .name('set-version')
    .description(
      'Update the Cargo workspace version, root package version and transport package versions.'
    )
    .argument('<version>', 'Semantic version to apply to the workspace.')
    .showHelpAfterError();

  await program.parseAsync(argv);
  const [raw] = program.processedArgs as [string | undefined];
  const version = raw?.trim();
  if (!version) {
    throw new Error('Missing required version argument.');
  }
  if (/\s/.test(version)) {
    throw new Error(`Invalid version "${version}": whitespace is not allowed.`);
  }
  return version;
}

async function updateCargoWorkspaceVersion(rootDir: string, version: string): Promise<void> {
  const manifestPath = join(rootDir, 'Cargo.toml');
  const raw = await readFile(manifestPath, 'utf8');
  const lines = raw.split('\n');
  let insideWorkspacePackage = false;
  let updated = false;

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (line === undefined) {
      continue;
    }
    if (line.trim() === '[workspace.package]') {
      insideWorkspacePackage = true;
      continue;
    }
    if (insideWorkspacePackage && /^\[.*\]$/.test(line.trim())) {
      break;
    }
    if (insideWorkspacePackage && /^\s*version\s*=/.test(line)) {
      lines[index] = `version = "${version}"`;
      updated = true;
      break;
    }
  }

  if (!updated) {
    throw new Error(`Failed to update workspace version in ${manifestPath}`);
  }

  await writeFile(manifestPath, `${lines.join('\n')}\n`, 'utf8');
  console.log(`cargo workspace: -> ${version}`);
}

async function updateRootPackageVersion(rootDir: string, version: string): Promise<void> {
  const packagePath = join(rootDir, 'package.json');
  const raw = await readFile(packagePath, 'utf8');
  const pkg = JSON.parse(raw) as PackageJson;
  const previous = pkg.version ?? '(undefined)';
  pkg.version = version;
  await writeFile(packagePath, `${JSON.stringify(pkg, null, 2)}\n`, 'utf8');
  console.log(`root package: ${previous} -> ${version}`);
}

async function updatePackageVersion(
  rootDir: string,
  packageDirName: string,
  version: string
): Promise<void> {
  const packagePath = join(rootDir, 'packages', packageDirName, 'package.json');
  const raw = await readFile(packagePath, 'utf8');
  const pkg = JSON.parse(raw) as PackageJson;

  if (!pkg.name) {
    throw new Error(`Missing package name in ${packagePath}`);
  }

  if (pkg.name === '@vulfram/transport-types') {
    throw new Error('transport-types must not be updated by this script.');
  }

  const previous = pkg.version ?? '(undefined)';
  pkg.version = version;

  await writeFile(packagePath, `${JSON.stringify(pkg, null, 2)}\n`, 'utf8');
  console.log(`${pkg.name}: ${previous} -> ${version}`);
}

async function main(): Promise<void> {
  const version = await parseVersionArg(process.argv);
  const rootDir = join(import.meta.dir, '..');

  await updateCargoWorkspaceVersion(rootDir, version);
  await updateRootPackageVersion(rootDir, version);

  for (const packageDirName of TARGET_PACKAGES) {
    await updatePackageVersion(rootDir, packageDirName, version);
  }

  console.log('Done.');
}

main().catch((error) => {
  console.error('[set-version] Failed:', error);
  process.exitCode = 1;
});
