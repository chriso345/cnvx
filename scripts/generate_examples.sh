#!/usr/bin/env bash
#
# Recursively scans an "examples" directory for .rs files and (re)writes the
# [[example]] blocks in Cargo.toml, based on each file's doc-comment.
#
# Usage:
#   ./generate_examples.sh [examples_dir] [Cargo.toml path]
#
# Defaults: examples_dir=examples, Cargo.toml=Cargo.toml (in cwd)

EXAMPLES_DIR="${1:-examples}"
CARGO_TOML="${2:-Cargo.toml}"
MARKER="## AUTO GENERATED EXAMPLES - DO NOT EDIT MANUALLY"

if [[ ! -d "$EXAMPLES_DIR" ]]; then
  echo "Error: examples directory '$EXAMPLES_DIR' not found" >&2
  exit 1
fi

if [[ ! -f "$CARGO_TOML" ]]; then
  echo "Error: '$CARGO_TOML' not found" >&2
  exit 1
fi

# Normalize a leading "./" so paths look like "examples/foo.rs" not "./examples/foo.rs"
normalize_path() {
  local p="$1"
  p="${p#./}"
  printf '%s' "$p"
}

declare -A seen_names # name -> file path, used for duplicate detection
entries=()            # collected [[example]] blocks

# Use find -print0 / read -d '' to safely handle any filenames (spaces, etc.)
while IFS= read -r -d '' file; do
  rel_path="$(normalize_path "$file")"
  base="$(basename "$rel_path")"
  name="${base%.rs}"

  if [[ -n "${seen_names[$name]+x}" ]]; then
    echo "Error: duplicate example name '$name' found at both '${seen_names[$name]}' and '$rel_path'" >&2
    exit 1
  fi
  seen_names["$name"]="$rel_path"

  # Grab the doc-comment "Features:" line, e.g. //! Features: `lp`, `milp`
  features_line="$(grep -m1 -E '^[[:space:]]*//!\s*Features:' "$file" || true)"

  features=()
  if [[ -n "$features_line" ]]; then
    while IFS= read -r feat; do
      [[ -n "$feat" ]] && features+=("$feat")
    done < <(grep -oE '`[^`]+`' <<<"$features_line" | tr -d '`')
  fi

  entry="[[example]]"$'\n'
  entry+="name = \"$name\""$'\n'
  entry+="path = \"$rel_path\""$'\n'

  if ((${#features[@]} > 0)); then
    feat_str=""
    for f in "${features[@]}"; do
      feat_str+="\"$f\", "
    done
    feat_str="${feat_str%, }"
    entry+="required-features = [$feat_str]"$'\n'
  fi

  entries+=("$entry")
done < <(find "$EXAMPLES_DIR" -type f -name '*.rs' -print0 | sort -z)

if ((${#entries[@]} == 0)); then
  echo "Warning: no .rs files found under '$EXAMPLES_DIR'" >&2
fi

# Join entries with a blank line between blocks
generated=""
for e in "${entries[@]}"; do
  generated+="$e"$'\n'
done
generated="${generated%$'\n'}"

# Build the new Cargo.toml
tmpfile="$(mktemp)"
if grep -qF "$MARKER" "$CARGO_TOML"; then
  awk -v marker="$MARKER" '
        found { next }
        { print }
        $0 == marker { found = 1 }
    ' "$CARGO_TOML" >"$tmpfile"
else
  cp "$CARGO_TOML" "$tmpfile"
  # Ensure exactly one blank line before the marker if the file is non-empty
  if [[ -s "$tmpfile" ]]; then
    printf '\n' >>"$tmpfile"
  fi
  printf '%s\n' "$MARKER" >>"$tmpfile"
fi

{
  cat "$tmpfile"
  printf '\n'
  printf '%s\n' "$generated"
} >"${CARGO_TOML}.new"

mv "${CARGO_TOML}.new" "$CARGO_TOML"
rm -f "$tmpfile"

echo "Wrote ${#entries[@]} example(s) to $CARGO_TOML"
