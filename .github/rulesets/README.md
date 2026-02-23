# Rulesets JSON

Arquivos prontos para criar rulesets via GitHub API (ou servir de base para import no UI).

- `branch-main-stable.json`: protege `main` e `stable/*` com PR + checks obrigatorios.
- `branch-promotion-channels.json`: protege `alpha/*`, `beta/*`, `release/*` e exige PR de promotion vindo de `main` ou `stable/*`.
- `tag-v-release.json`: protege tags `v*` contra update/delete.

## Uso rapido (GitHub CLI)

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

## Atualizacao

Para atualizar um ruleset existente, use `PATCH /repos/{owner}/{repo}/rulesets/{ruleset_id}` com o JSON ajustado.
