import { getRepoContext, githubRequest, requireEnv } from './api';

type CompareResponse = {
  commits: Array<{ sha: string }>;
};

type PullRequestSummary = {
  labels: Array<{ name: string }>;
  merged_at?: string | null;
  number: number;
  title: string;
  user: { login: string };
};

const CATEGORIES = [
  ['changelog:breaking', 'Breaking Changes'],
  ['changelog:feature', 'Features'],
  ['changelog:fix', 'Fixes'],
  ['changelog:performance', 'Performance'],
  ['changelog:docs', 'Documentation']
] as const;

async function collectPullsForCommit(config: {
  commitSha: string;
  owner: string;
  repo: string;
}): Promise<PullRequestSummary[]> {
  return githubRequest<PullRequestSummary[]>({
    path: `/repos/${config.owner}/${config.repo}/commits/${config.commitSha}/pulls`,
    headers: {
      Accept: 'application/vnd.github.groot-preview+json'
    }
  });
}

async function main(): Promise<void> {
  const { owner, repo } = getRepoContext();
  const previousTag = process.env.PREVIOUS_TAG?.trim() ?? '';
  const releaseTag = requireEnv('RELEASE_TAG');
  const packageVersion = requireEnv('PACKAGE_VERSION');
  const channel = requireEnv('CHANNEL');
  const branchRef = requireEnv('BRANCH_REF');
  const npmTag = requireEnv('NPM_TAG');
  const head = requireEnv('GITHUB_SHA');

  const categorized = new Map(
    CATEGORIES.map(([label, title]) => [label, { title, entries: [] as string[] }])
  );
  const uncategorized: string[] = [];
  const seenPrs = new Set<number>();

  if (previousTag) {
    const comparison = await githubRequest<CompareResponse>({
      path: `/repos/${owner}/${repo}/compare/${previousTag}...${head}`
    });

    for (const commit of comparison.commits) {
      const pulls = await collectPullsForCommit({ commitSha: commit.sha, owner, repo });
      for (const pr of pulls) {
        if (!pr.merged_at || seenPrs.has(pr.number)) {
          continue;
        }

        seenPrs.add(pr.number);
        const labels = new Set(pr.labels.map((label) => label.name));
        if (labels.has('skip-changelog') || labels.has('changelog:internal')) {
          continue;
        }

        const line = `- ${pr.title} (#${pr.number}) @${pr.user.login}`;
        let matched = false;
        for (const [label] of CATEGORIES) {
          if (!labels.has(label)) {
            continue;
          }
          categorized.get(label)?.entries.push(line);
          matched = true;
        }

        if (!matched) {
          uncategorized.push(line);
        }
      }
    }
  }

  const body: string[] = ['## Changes'];
  let hasChanges = false;
  for (const [, section] of categorized) {
    if (section.entries.length === 0) {
      continue;
    }
    hasChanges = true;
    body.push('', `### ${section.title}`, ...section.entries);
  }

  if (uncategorized.length > 0) {
    hasChanges = true;
    body.push('', '### Other Changes', ...uncategorized);
  }

  if (!hasChanges) {
    body.push('', '- No categorized changes were found for this release window.');
  }

  body.push(
    '',
    '## Release Metadata',
    `- Channel: \`${channel}\``,
    `- Source branch: \`${branchRef}\``,
    `- Git tag: \`${releaseTag}\``,
    `- Package version: \`${packageVersion}\``
  );
  if (previousTag) {
    body.push(`- Previous tag: \`${previousTag}\``);
  }

  body.push(
    '',
    '## Bind Groups',
    '- `ffi`',
    '- `napi`',
    '- `wasm`',
    '- `python`',
    '- `lua`',
    '',
    '## npm Packages',
    `- \`@vulfram/transport-browser\` via dist-tag \`${npmTag}\``,
    `- \`@vulfram/transport-bun\` via dist-tag \`${npmTag}\``,
    `- \`@vulfram/transport-napi\` via dist-tag \`${npmTag}\``,
    `- \`@vulfram/engine\` via dist-tag \`${npmTag}\``,
    `- \`@vulfram/gltf-loader\` via dist-tag \`${npmTag}\``,
    `- \`@vulfram/camera-control\` via dist-tag \`${npmTag}\``
  );

  await Bun.write('release-notes.md', `${body.join('\n')}\n`);
}

main().catch((error) => {
  console.error('[generate-release-notes] Failed:', error);
  process.exitCode = 1;
});
