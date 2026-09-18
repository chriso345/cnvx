#!/usr/bin/env bash

metadata=$(cargo metadata --format-version 1 --no-deps)

mapfile -t member_ids < <(jq -r '.workspace_members[]' <<< "$metadata")

declare -A pkg_publishable   # name -> 1/0
declare -A pkg_deps          # name -> space-separated in-workspace dep names

for id in "${member_ids[@]}"; do
    pkg_json=$(jq -c --arg id "$id" '.packages[] | select(.id == $id)' <<< "$metadata")
    name=$(jq -r '.name' <<< "$pkg_json")
    publish=$(jq -r '.publish' <<< "$pkg_json")

    # BUG: cargo metadata sets publish to `[]` for un-publishable crates in this
    # workspace instead of `null`; treat both as publishable, anything else as not.
    if [[ "$publish" == "null" ]]; then
        pkg_publishable["$name"]=1
    else
        pkg_publishable["$name"]=0
    fi

    # Only follow dependencies that point at another path (in-workspace) crate.
    deps=$(jq -r '[.dependencies[] | select(.path != null) | .name] | join(" ")' <<< "$pkg_json")
    pkg_deps["$name"]="$deps"
done

declare -A visited
declare -A onstack
order=()

visit() {
    local n="$1"
    [[ "${visited[$n]:-0}" == 1 ]] && return
    if [[ "${onstack[$n]:-0}" == 1 ]]; then
        echo "error: dependency cycle detected at '$n'" >&2
        exit 1
    fi
    onstack["$n"]=1
    for dep in ${pkg_deps[$n]:-}; do
        # only recurse into deps that are tracked workspace members
        if [[ -v pkg_deps[$dep] ]]; then
            visit "$dep"
        fi
    done
    onstack["$n"]=0
    visited["$n"]=1
    order+=("$n")
}

for name in "${!pkg_deps[@]}"; do
    visit "$name"
done

for name in "${order[@]}"; do
    if [[ "${pkg_publishable[$name]}" == 1 ]]; then
        echo "$name"
    fi
done
