#!/usr/bin/env bash

set -e

RUSTDOCFLAGS="-D warnings" cargo doc \
  --workspace \
  --no-deps \
  --document-private-items
