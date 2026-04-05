import { Command } from 'commander';

type ReleaseChannel = 'alpha' | 'beta' | 'next' | 'latest';

type ReleaseMeta = {
  artifactVersion: string;
  branchRef: string;
  channel: ReleaseChannel;
  npmTag: ReleaseChannel;
  packageVersion: string;
  releaseTag: string;
  versionCore: string;
};

type CliOptions = {
  githubOutput?: string;
  ref?: string;
};

function resolvePackageVersion(channel: ReleaseChannel, versionCore: string): string {
  if (channel === 'latest') {
    return versionCore;
  }
  return `${versionCore}-${channel}`;
}

function parseBranchRef(ref: string): ReleaseMeta {
  const normalized = ref.trim();
  const match = normalized.match(/^(alpha|beta|next|latest)\/v(\d+\.\d+\.\d+)$/);
  if (!match) {
    throw new Error(
      `Invalid release branch "${ref}". Expected alpha|beta|next|latest followed by /vX.Y.Z.`
    );
  }

  const channel = match[1] as ReleaseChannel;
  const versionCore = match[2]!;
  const packageVersion = resolvePackageVersion(channel, versionCore);

  return {
    artifactVersion: `v${versionCore}`,
    branchRef: normalized,
    channel,
    npmTag: channel,
    packageVersion,
    releaseTag: `v${packageVersion}`,
    versionCore
  };
}

function printMeta(meta: ReleaseMeta): void {
  for (const [key, value] of Object.entries(meta)) {
    console.log(`${key}=${value}`);
  }
}

async function parseOptions(): Promise<CliOptions> {
  const program = new Command();
  program
    .name('release-meta')
    .description('Resolve release metadata from a promotion branch name.')
    .option('--ref <branch>', 'Branch name like alpha/v0.22.0.')
    .option('--github-output <path>', 'Optional GitHub Actions output file path.');

  await program.parseAsync(process.argv);
  return program.opts<CliOptions>();
}

async function main(): Promise<void> {
  const options = await parseOptions();
  const ref =
    options.ref?.trim() ||
    process.env.GITHUB_BASE_REF ||
    process.env.GITHUB_REF_NAME ||
    process.env.RELEASE_BRANCH ||
    '';

  if (!ref) {
    throw new Error(
      'Missing release branch reference. Pass --ref or provide GITHUB_REF_NAME/GITHUB_BASE_REF.'
    );
  }

  const meta = parseBranchRef(ref);
  printMeta(meta);

  const outputPath = options.githubOutput?.trim() || process.env.GITHUB_OUTPUT || '';
  if (!outputPath) {
    return;
  }

  const lines = Object.entries(meta).map(([key, value]) => `${key}=${value}`);
  await Bun.write(outputPath, `${lines.join('\n')}\n`);
}

main().catch((error) => {
  console.error('[release-meta] Failed:', error);
  process.exitCode = 1;
});
