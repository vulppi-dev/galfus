# Rulesets JSON

Ready-to-use files for creating rulesets via GitHub API (or as a base for UI import).

- `branch-main-stable.json`: protects `main` and `stable/*` with PR + required checks.
- `branch-promotion-channels.json`: protects `alpha/*`, `beta/*`, `release/*` and requires promotion PRs coming from `main` or `stable/*`.
- `tag-v-release.json`: protects `v*` tags from update/delete.

## Quick Use (GitHub CLI)

```bash
# Branch ruleset
GH_REPO="vulppi-dev/vulfram" # owner/repo

gh api \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  "/repos/$GH_REPO/rulesets" \
  --input .github/rulesets/branch-main-stable.json

# Tag ruleset
gh api \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  "/repos/$GH_REPO/rulesets" \
  --input .github/rulesets/tag-v-release.json

# Promotion channels ruleset
gh api \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  "/repos/$GH_REPO/rulesets" \
  --input .github/rulesets/branch-promotion-channels.json
```

## Update

To update an existing ruleset, use `PATCH /repos/{owner}/{repo}/rulesets/{ruleset_id}` with the adjusted JSON.
