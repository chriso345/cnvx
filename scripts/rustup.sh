#!/usr/bin/env bash

set -e

if ! command -v rustup &>/dev/null; then
  echo "rustup is not installed. Please install it from https://rustup.rs/ and try again."
  exit 1
fi

if rustup toolchain list | grep --silent 'nightly'; then
  echo "Updating nightly toolchain..."
  rustup update nightly
else
  echo "Installing nightly toolchain..."
  rustup install nightly
fi
