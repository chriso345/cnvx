#!/usr/bin/env bash

set -e

# Checks the correct nightly toolchain is installed.
./scripts/rustup.sh

./scripts/lint.sh --pre-commit
./scripts/fmt.sh --pre-commit
./scripts/docs.sh --pre-commit
