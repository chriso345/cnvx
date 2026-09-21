#!/usr/bin/env bash

set -e

RUSTDOCFLAGS="-D warnings" cargo +nightly doc \
  --workspace                                 \
  --no-deps                                   \
  --document-private-items
