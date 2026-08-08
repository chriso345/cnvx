#!/usr/bin/env bash

set -e

if [ "$1" = "--pre-commit" ]; then
  if ! rustup toolchain list | grep --silent 'nightly'; then
    echo "Nightly toolchain is not installed. Please install it with 'rustup toolchain install nightly' and try again."
    exit 1
  fi
fi

cargo +nightly fmt --check
