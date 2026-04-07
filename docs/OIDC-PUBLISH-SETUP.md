# OIDC Publish Setup

This guide explains how to configure GitHub Actions OIDC publishing for npm and
for the JSR packages that are currently published by the workflow.

## What This Repository Publishes

npm publish flow currently targets:

- `@vulfram/transport-browser`
- `@vulfram/transport-bun`
- `@vulfram/transport-napi`
- `@vulfram/engine`
- `@vulfram/gltf-loader`
- `@vulfram/camera-control`

JSR publish flow currently targets:

- `@vulfram/transport-types`
- `@vulfram/transport-browser`
- `@vulfram/engine`
- `@vulfram/gltf-loader`
- `@vulfram/camera-control`

Manual JSR publish only:

- none

Excluded from JSR workflow:

- `@vulfram/transport-bun`
- `@vulfram/transport-napi`

The excluded packages remain out of JSR because their native multi-platform
artifacts make the publish payload too large for the registry.

The workflow file that publishes them is:

- `.github/workflows/build-bindings.yml`

## GitHub Actions Requirements

The workflow must run on GitHub-hosted runners and must request:

- `id-token: write`

This repository already configures that in:

- `.github/workflows/build-bindings.yml`

## npm Trusted Publishing Setup

Configure npm trusted publishing once for each npm package listed above.

For every package:

1. Open the package settings on npmjs.com.
2. Go to the `Trusted Publisher` section.
3. Choose `GitHub Actions`.
4. Fill the form with:
   - Organization or user: `vulppi-dev`
   - Repository: `vulfram`
   - Workflow filename: `build-bindings.yml`
   - Environment name: leave empty unless you later protect publishing with a GitHub Environment
5. Save the trusted publisher configuration.

Important notes:

- npm matches these values exactly and case-sensitively.
- The package `repository.url` must point to `git+https://github.com/vulppi-dev/vulfram.git`.
- Trusted publishing replaces the old long-lived publish token flow for `npm publish`.
- With trusted publishing, npm generates provenance automatically. The workflow still uses `--provenance` explicitly for clarity.

## JSR OIDC Setup

Configure JSR package linking once for each JSR package published by the
workflow.

`@vulfram/transport-bun` and `@vulfram/transport-napi` are intentionally
excluded from the JSR workflow because their native multi-platform artifacts
make the publish payload too large for JSR.

For every package:

1. Open the package page on `jsr.io`.
2. Go to the package `Settings`.
3. In the GitHub repository/link section, link the package to:
   - owner: `vulppi-dev`
   - repository: `vulfram`
4. Save the link.

After the package is linked, `npx jsr publish` in GitHub Actions can
authenticate through OIDC without a token.

Important notes:

- JSR uses the package `jsr.json` version.
- This repository synchronizes those versions through `bun run version -- <version>`.
- The workflow checks whether the mapped version already exists and skips it
  without failing.

## Repository Secrets

After switching npm publishing to trusted publishing, this flow no longer requires:

- `NPM_TOKEN`

If the repository does not install private npm dependencies, no npm auth secret is needed for publishing.

## Verification Checklist

Before promoting a release branch, verify:

1. The package exists on npm or JSR already, if the registry requires prior ownership setup.
2. The package is linked to the correct GitHub repository in the registry UI.
3. The workflow filename registered in npm is exactly `build-bindings.yml`.
4. The package manifest still contains the correct `repository.url`.
5. The workflow still has `id-token: write`.

## Release Flow Summary

1. Push or dispatch from a promotion branch like `alpha/v0.22.0`.
2. The workflow resolves the release version.
3. Rust artifacts are built and grouped.
4. GitHub Release assets are published.
5. npm packages are published through OIDC trusted publishing in a dedicated job.
6. JSR packages are published through OIDC in a separate dedicated job.
