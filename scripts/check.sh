#!/usr/bin/env bash
set -euo pipefail

cargo check --lib \
  && cargo check -p vulfram-runtime --lib \
  && cargo run --bin wgsl_check \
  && cargo test -p vulfram-runtime --lib \
  && cargo fmt --all
