#!/usr/bin/env bash

metadata=$(cargo metadata --format-version 1 --no-deps)

mapfile -t packages < <(
    jq -r '.workspace_members[]' <<< "$metadata"
)

# BUG: There seems to be a bug in cargo metadata, when publish = true in Cargo.toml
#  it is set to `publish: []`.
for id in "${packages[@]}"; do
    jq -r --arg id "$id" \
    '.packages[] | select(.id == $id) | select(.publish == null) | .name' \
        <<< "$metadata"
done
