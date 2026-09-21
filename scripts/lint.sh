#!/usr/bin/env bash

set -e

run_clippy() {
  local fix_mode=$1
  cargo +nightly clippy --workspace --all-targets \
        $1 --                                     \
        -D warnings
}

if [ "$1" = "--pre-commit" ]; then
  if ! rustup toolchain list | grep --silent 'nightly'; then
    echo "Nightly toolchain is not installed. Please install it with 'rustup toolchain install nightly' and try again."
    exit 1
  fi
elif [ "$1" = "--fix" ]; then
  run_clippy "--fix --allow-dirty"
fi

run_clippy
