#!/usr/bin/env bash
#
# Recursively scans an "examples" directory for .rs files, runs each one
# (with --features derived from its doc-comment "Features:" line, if any)
# via `cargo run --example <name>`, and rewrites the file's
# "// Expected output:" comment block to match the actual stdout produced.
#
# Usage:
#   ./update_examples.sh [-d examples_dir] [-m manifest_path] [-n] [name ...]
#
#   -d DIR    examples directory to scan (default: examples)
#   -m PATH   path to Cargo.toml / manifest to pass as --manifest-path
#             (default: none, cargo uses the one in the cwd)
#   -n        dry run: show what would change but don't write any files
#   name ...  optional list of example names (file names without .rs) to
#             update. If omitted, every .rs file found under DIR is checked.
#
# Exit code is the number of examples that failed to run (0 means every
# example ran successfully, regardless of whether its output changed).

set -uo pipefail

EXAMPLES_DIR="examples"
MANIFEST_PATH=""
DRY_RUN=0

while getopts ":d:m:nh" opt; do
    case "$opt" in
        d) EXAMPLES_DIR="$OPTARG" ;;
        m) MANIFEST_PATH="$OPTARG" ;;
        n) DRY_RUN=1 ;;
        h)
            sed -n '2,40p' "$0"
            exit 0
            ;;
        \?)
            echo "Unknown option: -$OPTARG" >&2
            exit 2
            ;;
        :)
            echo "Option -$OPTARG requires an argument" >&2
            exit 2
            ;;
    esac
done
shift $((OPTIND - 1))

WANTED_NAMES=("$@")

if [[ ! -d "$EXAMPLES_DIR" ]]; then
    echo "Error: examples directory '$EXAMPLES_DIR' not found" >&2
    exit 1
fi

# Print the features (comma-separated) declared via a
#   //! Features: `foo`, `bar`
# doc-comment line, or nothing if there isn't one.
extract_features() {
    local file="$1"
    local features_line
    features_line="$(grep -m1 -E '^[[:space:]]*//!\s*Features:' "$file" || true)"
    [[ -z "$features_line" ]] && return 0

    local feats=()
    while IFS= read -r feat; do
        [[ -n "$feat" ]] && feats+=("$feat")
    done < <(grep -oE '`[^`]+`' <<< "$features_line" | tr -d '`')

    local IFS=,
    printf '%s' "${feats[*]}"
}

# Print the expected-output block that follows a
#   // Expected output:
# comment, with the leading indentation / "// " prefix stripped, and any
# leading blank separator line (the "//" right after the header) dropped.
extract_expected_output() {
    local file="$1"
    awk '
        BEGIN { found = 0 }
        found == 0 {
            if ($0 ~ /Expected output:/) { found = 1 }
            next
        }
        {
            if ($0 ~ /^[[:space:]]*\/\//) {
                line = $0
                sub(/^[[:space:]]*\/\/[[:space:]]?/, "", line)
                print line
            } else {
                exit
            }
        }
    ' "$file" | awk '
        started == 0 && NF == 0 { next }
        { started = 1; print }
    '
}

name_is_wanted() {
    local name="$1"
    (( ${#WANTED_NAMES[@]} == 0 )) && return 0
    local w
    for w in "${WANTED_NAMES[@]}"; do
        [[ "$w" == "$name" ]] && return 0
    done
    return 1
}

# Rewrite the "Expected output:" block in $1 to contain $2 (a possibly
# multi-line string), preserving the original indentation. Returns 1 (and
# leaves the file untouched) if no such block is found in the file.
rewrite_expected_block() {
    local file="$1"
    local new_output="$2"

    local header_idx
    header_idx="$(grep -n -m1 -E '^[[:space:]]*//[[:space:]]*Expected output:' "$file" | cut -d: -f1 || true)"
    if [[ -z "$header_idx" ]]; then
        return 1
    fi

    local indent
    indent="$(sed -n "${header_idx}p" "$file" | sed -E 's#^([[:space:]]*)//.*#\1#')"

    # Find the first line after the header that is NOT a "//" comment line;
    # that marks the end (exclusive) of the existing block.
    local end_idx
    end_idx="$(awk -v start="$((header_idx + 1))" '
        NR >= start && $0 !~ /^[[:space:]]*\/\// { print NR; exit }
    ' "$file")"

    mapfile -t lines < "$file"
    local total="${#lines[@]}"
    [[ -z "$end_idx" ]] && end_idx=$((total + 1))

    local new_lines=()
    local i
    # Keep everything up to and including the header line itself.
    for (( i = 0; i < header_idx; i++ )); do
        new_lines+=("${lines[$i]}")
    done

    new_lines+=("${indent}//")
    while IFS= read -r out_line; do
        if [[ -z "$out_line" ]]; then
            new_lines+=("${indent}//")
        else
            new_lines+=("${indent}// ${out_line}")
        fi
    done <<< "$new_output"

    # Keep everything from the terminating line onward.
    for (( i = end_idx - 1; i < total; i++ )); do
        new_lines+=("${lines[$i]}")
    done

    if (( DRY_RUN == 0 )); then
        printf '%s\n' "${new_lines[@]}" > "${file}.tmp.$$"
        mv "${file}.tmp.$$" "$file"
    fi
    return 0
}

declare -A seen_names
files=()

while IFS= read -r -d '' file; do
    base="$(basename "$file")"
    name="${base%.rs}"

    if [[ -n "${seen_names[$name]+x}" ]]; then
        echo "Error: duplicate example name '$name' found at both '${seen_names[$name]}' and '$file'" >&2
        exit 1
    fi
    seen_names["$name"]="$file"

    name_is_wanted "$name" && files+=("$file")
done < <(find "$EXAMPLES_DIR" -type f -name '*.rs' -print0 | sort -z)

if (( ${#files[@]} == 0 )); then
    echo "No matching .rs files found under '$EXAMPLES_DIR'" >&2
    exit 1
fi

for w in "${WANTED_NAMES[@]:-}"; do
    [[ -z "$w" ]] && continue
    if [[ -z "${seen_names[$w]+x}" ]]; then
        echo "Warning: no example named '$w' found under '$EXAMPLES_DIR'" >&2
    fi
done

updated_count=0
unchanged_count=0
skipped_count=0
error_count=0
errored_names=()

for file in "${files[@]}"; do
    base="$(basename "$file")"
    name="${base%.rs}"

    features="$(extract_features "$file")"

    cmd=(cargo run --quiet --example "$name")
    [[ -n "$MANIFEST_PATH" ]] && cmd+=(--manifest-path "$MANIFEST_PATH")
    [[ -n "$features" ]] && cmd+=(--features "$features")

    echo "=== $name ($file) ==="
    if [[ -n "$features" ]]; then
        echo "  features: $features"
    fi
    echo "  cmd: ${cmd[*]}"

    actual="$("${cmd[@]}" 2>/tmp/update_examples_stderr.$$)"
    status=$?

    if (( status != 0 )); then
        echo "  ERROR (exit code $status), leaving file untouched"
        echo "  --- stderr ---"
        sed 's/^/    /' /tmp/update_examples_stderr.$$
        error_count=$((error_count + 1))
        errored_names+=("$name")
        echo
        rm -f /tmp/update_examples_stderr.$$
        continue
    fi
    rm -f /tmp/update_examples_stderr.$$

    current_expected="$(extract_expected_output "$file")"
    norm_actual="$(printf '%s' "$actual" | sed -e 's/[[:space:]]*$//')"
    norm_current="$(printf '%s' "$current_expected" | sed -e 's/[[:space:]]*$//')"

    if [[ "$norm_actual" == "$norm_current" ]]; then
        echo "  unchanged"
        unchanged_count=$((unchanged_count + 1))
        echo
        continue
    fi

    if ! rewrite_expected_block "$file" "$norm_actual"; then
        echo "  SKIPPED (no 'Expected output:' block found to update)"
        skipped_count=$((skipped_count + 1))
        echo
        continue
    fi

    if (( DRY_RUN == 1 )); then
        echo "  would update (dry run) -- diff (old vs new):"
    else
        echo "  updated -- diff (old vs new):"
    fi
    diff <(printf '%s\n' "$norm_current") <(printf '%s\n' "$norm_actual") | sed 's/^/    /'
    updated_count=$((updated_count + 1))
    echo
done

echo "==================================================="
echo "Checked $((updated_count + unchanged_count + skipped_count + error_count)) example(s):" \
     "$updated_count updated, $unchanged_count unchanged, $skipped_count skipped, $error_count errored"
if (( DRY_RUN == 1 )) && (( updated_count > 0 )); then
    echo "(dry run: no files were actually written)"
fi
if (( error_count > 0 )); then
    echo "Errored: ${errored_names[*]}"
fi

exit "$error_count"
