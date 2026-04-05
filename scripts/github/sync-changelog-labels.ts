import { getRepoContext, paginateGitHub, readEventPayload, githubRequest } from './api';

type PullRequestLabel = {
  name: string;
};

type PullRequestPayload = {
  pull_request?: {
    body?: string | null;
    labels: PullRequestLabel[];
    number: number;
  };
};

const MANAGED_LABELS = [
  'changelog:breaking',
  'changelog:feature',
  'changelog:fix',
  'changelog:performance',
  'changelog:docs',
  'changelog:internal'
] as const;

type RepoLabel = {
  name: string;
};

async function main(): Promise<void> {
  const payload = await readEventPayload<PullRequestPayload>();
  const pr = payload.pull_request;
  if (!pr) {
    return;
  }

  const checked = new Set<string>();
  const body = pr.body ?? '';
  const checkedPattern = /-\s*\[(x|X)\]\s*`([^`]+)`/g;
  let match: RegExpExecArray | null;
  while ((match = checkedPattern.exec(body)) !== null) {
    const label = match[2]?.trim();
    if (label && MANAGED_LABELS.includes(label as (typeof MANAGED_LABELS)[number])) {
      checked.add(label);
    }
  }

  const { owner, repo } = getRepoContext();
  const currentPrLabels = new Set(pr.labels.map((label) => label.name));
  const existingRepoLabels = await paginateGitHub<RepoLabel>(`/repos/${owner}/${repo}/labels`);
  const repoLabelSet = new Set(existingRepoLabels.map((label) => label.name));

  const toAdd = [...checked].filter(
    (label) => repoLabelSet.has(label) && !currentPrLabels.has(label)
  );
  const toRemove = MANAGED_LABELS.filter(
    (label) => repoLabelSet.has(label) && currentPrLabels.has(label) && !checked.has(label)
  );

  if (toAdd.length > 0) {
    await githubRequest({
      method: 'POST',
      path: `/repos/${owner}/${repo}/issues/${pr.number}/labels`,
      body: { labels: toAdd }
    });
  }

  for (const label of toRemove) {
    try {
      await githubRequest({
        method: 'DELETE',
        path: `/repos/${owner}/${repo}/issues/${pr.number}/labels/${encodeURIComponent(label)}`
      });
    } catch (error) {
      const status = (error as { status?: number }).status;
      if (status !== 404) {
        throw error;
      }
    }
  }

  console.log(`changelog labels added=[${toAdd.join(', ')}], removed=[${toRemove.join(', ')}]`);
}

main().catch((error) => {
  console.error('[sync-changelog-labels] Failed:', error);
  process.exitCode = 1;
});
