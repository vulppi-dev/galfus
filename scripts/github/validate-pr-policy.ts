import { getRepoContext, githubRequest, readEventPayload } from './api';

type PullRequestPayload = {
  pull_request?: {
    base: { ref: string };
    body?: string | null;
    draft: boolean;
    head: { ref: string };
    labels: Array<{ name: string }>;
    number: number;
    title?: string | null;
  };
};

type IssueResponse = {
  pull_request?: unknown;
};

type PullResponse = {
  labels: Array<{ name: string }>;
};

const CHANGELOG_LABELS = [
  'changelog:breaking',
  'changelog:feature',
  'changelog:fix',
  'changelog:performance',
  'changelog:docs',
  'changelog:internal'
];

async function main(): Promise<void> {
  const payload = await readEventPayload<PullRequestPayload>();
  const pr = payload.pull_request;
  if (!pr || pr.draft) {
    return;
  }

  const errors: string[] = [];
  const title = (pr.title ?? '').trim();
  const semanticTitle = /^(feat|fix|perf|docs|chore|refactor|test|build|ci)(\([^)]+\))?!?:\s+.+$/;
  if (!semanticTitle.test(title)) {
    errors.push(
      'Invalid PR title. Use semantic format: type(scope): summary (types: feat, fix, perf, docs, chore, refactor, test, build, ci).'
    );
  }

  const body = pr.body ?? '';
  const linkedTargetRegex =
    /\b(?:close|closes|closed|fix|fixes|fixed|resolve|resolves|resolved)\s+(#\d+|https:\/\/github\.com\/[^\s)]+\/[^\s)]+\/(?:issues|pull)\/\d+)\b/gi;
  const linkedIssues: Array<{ issueNumber: number; owner: string; raw: string; repo: string }> = [];

  for (const match of body.matchAll(linkedTargetRegex)) {
    const target = match[1];
    if (!target) {
      continue;
    }

    const shortRef = /^#(\d+)$/i.exec(target);
    if (shortRef) {
      const { owner, repo } = getRepoContext();
      linkedIssues.push({
        owner,
        repo,
        issueNumber: Number(shortRef[1]),
        raw: target
      });
      continue;
    }

    const urlRef = /^https:\/\/github\.com\/([^/\s]+)\/([^/\s]+)\/(issues|pull)\/(\d+)$/i.exec(
      target
    );
    if (!urlRef) {
      errors.push(`Invalid linked issue reference: "${target}".`);
      continue;
    }

    const [, owner, repo, kind, number] = urlRef;
    if (kind?.toLowerCase() !== 'issues') {
      errors.push(`Linked reference "${target}" must point to an issue, not a pull request.`);
      continue;
    }

    linkedIssues.push({
      owner: owner!,
      repo: repo!,
      issueNumber: Number(number),
      raw: target
    });
  }

  const seen = new Set<string>();
  for (const ref of linkedIssues) {
    const key = `${ref.owner}/${ref.repo}#${ref.issueNumber}`;
    if (seen.has(key)) {
      continue;
    }
    seen.add(key);

    try {
      const issue = await githubRequest<IssueResponse>({
        path: `/repos/${ref.owner}/${ref.repo}/issues/${ref.issueNumber}`
      });
      if (issue.pull_request) {
        errors.push(
          `Linked reference "${ref.raw}" resolves to PR #${ref.issueNumber}, not an issue.`
        );
      }
    } catch (error) {
      const status = (error as { status?: number }).status;
      if (status === 404) {
        errors.push(`Linked issue "${ref.raw}" was not found.`);
        continue;
      }
      throw error;
    }
  }

  const { owner, repo } = getRepoContext();
  const currentPr = await githubRequest<PullResponse>({
    path: `/repos/${owner}/${repo}/pulls/${pr.number}`
  });
  const labels = currentPr.labels.map((label) => label.name);
  if (!labels.some((name) => CHANGELOG_LABELS.includes(name))) {
    errors.push(`Missing changelog label. Add one of: ${CHANGELOG_LABELS.join(', ')}`);
  }

  const baseRef = pr.base.ref;
  const headRef = pr.head.ref;
  const isPromotionBase =
    /^alpha\/.+$/.test(baseRef) ||
    /^beta\/.+$/.test(baseRef) ||
    /^next\/.+$/.test(baseRef) ||
    /^latest\/.+$/.test(baseRef);

  if (isPromotionBase) {
    const isMain = headRef === 'main';
    const isStable = /^stable\/.+$/.test(headRef);
    if (!isMain && !isStable) {
      errors.push(
        `Invalid promotion source "${headRef}". Allowed sources for ${baseRef}: main or stable/*`
      );
    }
  }

  if (errors.length > 0) {
    throw new Error(errors.map((message) => `- ${message}`).join('\n'));
  }
}

main().catch((error) => {
  console.error('[validate-pr-policy] Failed:', error);
  process.exitCode = 1;
});
