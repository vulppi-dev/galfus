# CI R2 Publish

This project publishes build outputs directly to Cloudflare R2 from GitHub Actions.

## Branch convention

Publishing metadata is extracted from the branch name:

- `alpha/vX.Y` or `alpha/vX.Y.Z`
- `beta/vX.Y` or `beta/vX.Y.Z`
- `release/vX.Y` or `release/vX.Y.Z`

Where:

- `channel`: `alpha`, `beta`, `release`
- `version`: `vX.Y` or `vX.Y.Z`

## Bucket layout

All published files use this prefix:

- `v1/{channel}/{version}/{binding}/{platform}/`

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

## Required repository secrets

- `R2_ACCESS_KEY_ID`
- `R2_SECRET_ACCESS_KEY`
- `R2_BUCKET`
- `R2_ENDPOINT`

`R2_ENDPOINT` must be the S3-compatible endpoint URL for the target bucket/account.

## Publish behavior

- `push` on version branches builds and uploads to R2.
- `pull_request` runs build/validation but does not upload.
