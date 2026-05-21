type GitHubError = Error & {
  status?: number;
};

export function requireEnv(name: string): string {
  const value = process.env[name]?.trim();
  if (!value) {
    throw new Error(`Missing required environment variable: ${name}`);
  }
  return value;
}

export function getRepoContext(): { owner: string; repo: string } {
  const repository = requireEnv('GITHUB_REPOSITORY');
  const [owner, repo] = repository.split('/');
  if (!owner || !repo) {
    throw new Error(`Invalid GITHUB_REPOSITORY value: ${repository}`);
  }
  return { owner, repo };
}

export async function readEventPayload<T>(): Promise<T> {
  const eventPath = requireEnv('GITHUB_EVENT_PATH');
  return Bun.file(eventPath).json() as Promise<T>;
}

export async function githubRequest<T>(config: {
  path: string;
  method?: 'GET' | 'POST' | 'DELETE';
  body?: unknown;
  headers?: Record<string, string>;
}): Promise<T> {
  const token = requireEnv('GITHUB_TOKEN');
  const response = await fetch(`https://api.github.com${config.path}`, {
    method: config.method ?? 'GET',
    headers: {
      Accept: 'application/vnd.github+json',
      Authorization: `Bearer ${token}`,
      'User-Agent': 'galfus-bun-workflow',
      ...(config.body ? { 'Content-Type': 'application/json' } : {}),
      ...config.headers
    },
    body: config.body ? JSON.stringify(config.body) : undefined
  });

  if (!response.ok) {
    const text = await response.text();
    const error = new Error(
      `GitHub API request failed (${response.status}) ${config.method ?? 'GET'} ${config.path}: ${text}`
    ) as GitHubError;
    error.status = response.status;
    throw error;
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return (await response.json()) as T;
}

export async function paginateGitHub<T>(path: string): Promise<T[]> {
  const items: T[] = [];
  let page = 1;

  for (;;) {
    const separator = path.includes('?') ? '&' : '?';
    const batch = await githubRequest<T[]>({
      path: `${path}${separator}per_page=100&page=${page}`
    });
    items.push(...batch);
    if (batch.length < 100) {
      return items;
    }
    page += 1;
  }
}
