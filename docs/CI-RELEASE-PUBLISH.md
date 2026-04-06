# CI Release Publish

This project publishes binding outputs from GitHub Actions to GitHub Releases
and npm packages. JSR publishing is temporarily disabled in the release
workflow.

## Branch convention

Publishing metadata is extracted from the branch name:

- `alpha/vX.Y.Z`
- `beta/vX.Y.Z`
- `next/vX.Y.Z`
- `latest/vX.Y.Z`

Where:

- `channel`: `alpha`, `beta`, `next`, `latest`
- package version:
  - `alpha/vX.Y.Z` -> `X.Y.Z-alpha`
  - `beta/vX.Y.Z` -> `X.Y.Z-beta`
  - `next/vX.Y.Z` -> `X.Y.Z-next`
  - `latest/vX.Y.Z` -> `X.Y.Z`
- GitHub release tag:
  - `alpha` -> `vX.Y.Z-alpha`
  - `beta` -> `vX.Y.Z-beta`
  - `next` -> `vX.Y.Z-next`
  - `latest` -> `vX.Y.Z`

Current `platform` labels:

- `windows-x64`
- `windows-arm64`
- `linux-x64`
- `linux-arm64`
- `macos-x64`
- `macos-arm64`
- `browser` (for `wasm` binding)

Each folder contains:

- the compiled binary (`.dll`, `.so`, `.dylib`, `.node`, `.wasm`)
- `<artifact>.sha256`
- `manifest.json`

## GitHub Release assets

Each promotion branch publish also creates or updates a GitHub Release with grouped bind assets:

- `ffi`
- `napi`
- `wasm`
- `python`
- `lua`

Each group is uploaded as:

- `vulfram-<binding>-<package-version>.zip`
- `vulfram-<binding>-<package-version>.zip.sha256`

## npm publish behavior

After the GitHub Release is published, the npm job syncs transport artifacts and publishes:

- `@vulfram/transport-browser`
- `@vulfram/transport-bun`
- `@vulfram/transport-napi`
- `@vulfram/engine`
- `@vulfram/gltf-loader`
- `@vulfram/camera-control`

npm dist-tags are mapped directly from the branch channel:

- `alpha` -> `alpha`
- `beta` -> `beta`
- `next` -> `next`
- `latest` -> `latest`

npm publishing uses GitHub Actions OIDC trusted publishing and provenance.

## JSR publish behavior

The JSR publish job is temporarily disabled in the workflow.

Reason: the current publish payload is about 90 MB, while JSR currently accepts
up to 20 MB per package version.

`@vulfram/transport-types` is intentionally excluded from automatic publishing
and stays manual because its version is stable and no longer changes in the
normal release flow.

The temporary JSR disablement will be revisited after the package layout is
adjusted to fit the current registry limit.

## OIDC setup guide

- [OIDC-PUBLISH-SETUP.md](OIDC-PUBLISH-SETUP.md)

## Publish behavior

- `push` on promotion branches builds, publishes the grouped GitHub Release, and
  runs the npm publish job.
- `pull_request` runs build/validation and packages grouped bind archives without publishing.
- `workflow_dispatch` follows the same publish path as `push`, as long as it runs from a valid
  promotion branch.
